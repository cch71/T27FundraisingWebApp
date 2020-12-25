import React, { useState, useEffect } from "react";
import NavBar from "../components/navbar";
import { navigate } from "gatsby";
import {orderDb, OrderListItem} from "../js/ordersdb";
import currency from "currency.js";
import auth from "../js/auth"
import jQuery from 'jquery';
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config";
import bootstrapIconSprite from "bootstrap-icons/bootstrap-icons.svg";
import * as bs from 'bootstrap/dist/js/bootstrap.min.js'
const addOrderImg = bootstrapIconSprite + "#plus-square-fill";
const trashImg = bootstrapIconSprite + "#trash";
const pencilImg = bootstrapIconSprite + "#pencil";
const exportImg = bootstrapIconSprite + "#cloud-download";
const reportSettingsImg = bootstrapIconSprite + "#gear";

const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });
const dlgIdRoot = 'reportViewSettingsDlg';
let reportSettingsDlg = undefined;


////////////////////////////////////////////////////////////////////
//
class ReportViews {
	private currentView_: string = "";
	private currentDataTable_: any = undefined;
	private currentQueryResults_: Array<OrderListItem<string>> = undefined;

	////////////////////////////////////////////////////////////////////
	//
	show(view: string, frConfig: FundraiserConfig, userId: string|undefined) {
		const asyncShow = async () => {
			if (jQuery.fn.dataTable.isDataTable( '#orderListTable')) {
				if (view === this.currentView_) { return; }

				jQuery('#orderListTable').DataTable().clear();
				jQuery('#orderListTable').DataTable().destroy();
				jQuery('#orderListTable').empty();
				delete this.currentDataTable_;
				delete this.currentQueryResults_;
			}

			console.log(`Current View: ${this.currentView_} New View: ${view}`);
			this.currentView_ = 'Default';

			if(typeof this[`show${view}`] === 'function') {
				this[`show${view}`](frConfig, userId);
			} else {
				throw new Error(`Report View Type: ${view} not found`);
			}

			const spinnerElm = document.getElementById('orderLoadingSpinner');
			if (spinnerElm) {
				spinnerElm.className = "d-none";
			}
		}

		asyncShow()
			.then(()=>{})
			.catch((err: any)=>{
				if ('Invalid Session'===err) {
					navigate('/signon/')
				} else {
					const errStr = `Failed creating order list: ${JSON.stringify(err)}`;
					console.log(errStr);
					alert(errStr);
					throw err;
				}
			});
	}


	////////////////////////////////////////////////////////////////////
	//
	private getActionButtons(order: any) {
		return(
			`<div>` +
			`<button type="button" class="btn btn-outline-info me-1 order-edt-btn">` +
			`<svg class="bi" fill="currentColor"><use xlink:href="${pencilImg}" /></svg></button>` +
			`<button type="button" class="btn btn-outline-danger order-edt-btn">` +
			`<svg class="bi" fill="currentColor"><use xlink:href="${trashImg}" /></svg></button>` +
			`</div>`
		);
	}

	////////////////////////////////////////////////////////////////////
	//
	private registerActionButtonHandlers() {
		// Handle on Edit Scenario
		jQuery('#orderListTable').find('.btn-outline-info').on('click', (event: any)=>{
			const parentTr = jQuery(event.currentTarget).parents('tr');
			const row = this.currentDataTable_.row(parentTr);
			const orderId = row.data()[0];

			console.log(`Editing order for ${orderId}`);
			orderDb.setActiveOrder(); // Reset active order to let order edit for set it
			navigate('/order_step_1/', {state: {editOrderId: orderId}});
		});

		// Handle On Delete Scenario
		jQuery('#orderListTable').find('.btn-outline-danger').on('click', (event: any)=>{
			const parentTr = jQuery(event.currentTarget).parents('tr');
			const row = this.currentDataTable_.row(parentTr);
			const orderId = row.data()[0];

			console.log(`Deleting order for ${orderId}`);
			jQuery('#confirmDeleteOrderInput').val('');
			parentTr.find('button').attr("disabled", true);

			const dlgElm = document.getElementById('deleteOrderDlg');
			const delOrderDlg = new bs.Modal(dlgElm, {
				backdrop: true,
				keyboard: true,
				focus: true
			});

			jQuery('#deleteDlgBtn')
				.prop("disabled",true)
				.off('click')
				.click(
					(event: any)=>{
						console.log(`Delete confirmed for: ${orderId}`);
						delOrderDlg.hide();
						orderDb.deleteOrder(orderId).then(()=>{
							row.remove().draw();
						}).catch((err: any)=>{
							alert(`Failed to delete order: ${orderId}: ${err.message}`);
							parentTr.find('button').attr("disabled", false);
						});
					}
				);

			const dlgHandler = (event)=>{
				parentTr.find('button').attr("disabled", false);
				dlgElm.removeEventListener('hidden.bs.modal', dlgHandler);
			};
			dlgElm.addEventListener('hidden.bs.modal', dlgHandler);

			delOrderDlg.show();
		});
	}

	////////////////////////////////////////////////////////////////////
	//
	private async showDefault(frConfig: FundraiserConfig, userId: string|undefined) {

		if (!userId) { userId = auth.getCurrentUserId(); }

		// Build query fields
		const fieldNames = ["orderId", "firstName", "lastName"];
		if ('mulch' === frConfig.kind()) {
			fieldNames.push("deliveryId");
			fieldNames.push("products.spreading");
		}

		if ('any'===userId) {
			fieldNames.push("orderOwner");
		}


		this.currentQueryResults_ = await orderDb.query({fields: fieldNames, orderOwner: userId});
		const orders = this.currentQueryResults_;
		console.log(`Default Orders Page: ${JSON.stringify(orders)}`);

		// Fill out rows of data
		const orderDataSet = [];
		for (const order of orders) {
			const nameStr = `${order.firstName}, ${order.lastName}`;
			const ownerId = ('any'===userId)?order.orderOwner:userId;
			const orderDataItem = [order.orderId, ownerId, nameStr];
			if ('mulch' === frConfig.kind()) {
				if (order.deliveryId) {
					orderDataItem.push(frConfig.deliveryDateFromId(order.deliveryId));
				} else {
					orderDataItem.push('');
				}
				orderDataItem.push((order.products?.spreading?"Yes":"No"));
			}
			orderDataItem.push(this.getActionButtons(order));
			orderDataSet.push(orderDataItem);
		}


		const tableColumns = [
			{
				title: "OrderId",
				visible: false
			},
			{
				title: "OrderOwnerId",
				visible: false
			},
			{
				title: "Name",
				className: "all"
			}
		];

		if ('mulch' === frConfig.kind()) {
			tableColumns.push({ title: "Delivery Date" });
			tableColumns.push({ title: "Spreading" });
		}

		tableColumns.push({
			title: "Actions",
			"orderable": false,
			className: "all"
		});

		this.currentDataTable_ = jQuery('#orderListTable').DataTable({
			data: orderDataSet,
			paging: false,
			bInfo : false,
			columns: tableColumns
		});

		this.registerActionButtonHandlers();
	}

	////////////////////////////////////////////////////////////////////
	//
	private async showFull(frConfig: FundraiserConfig, userId: string|undefined) {

		if (!userId) { userId = auth.getCurrentUserId(); }

		this.currentQueryResults_ = await orderDb.query();
		const orders = this.currentQueryResults_;

		console.log(`Full Orders Page: ${JSON.stringify(orders)}`);

		const getVal = (fld: any|undefined, dflt: any|undefined)=>{
			if (undefined===fld) {
				if (undefined===dflt) {
					return '';
				} else {
					return `${dflt}`;
				}
			} else {
				return `${fld}`;
			}
		};

		// Fill out rows of data
		const orderDataSet = [];
		for (const order of orders) {
			const nameStr = `${order.firstName}, ${order.lastName}`;

			let orderDataItem = [
				order.orderId,
				order.orderOwner,
				nameStr,
				order.phone,
				getVal(order.email),
				order.addr1,
				getVal(order.addr2),
				(order.deliveryId?frConfig.deliveryDateFromId(order.deliveryId):'')
			];

			if ('mulch' === frConfig.kind()) {
				orderDataItem.push(order.neighborhood);
				orderDataItem.push(getVal(order.products?.spreading, 0));
				orderDataItem.push(getVal(order.products?.bags, 0));
			} else {
				//TODO:  Add Products stuff like city, state, zip
			}

			orderDataItem = orderDataItem.concat([
				getVal(order.specialInstructions),
				USD(order.donation).format(),
				USD(order.cashPaid).format(),
				USD(order.checkPaid).format(),
				getVal(order.checkNums),
				USD(order.totalAmt).format(),
				(order.isValidated?"True":"False")
			]);

			orderDataItem.push(this.getActionButtons(order));
			orderDataSet.push(orderDataItem);
		}


		let tableColumns = [
			{ title: "OrderId", visible: false },
			{ title: "OrderOwnerId", visible: false },
			{ title: "Name" },
			{ title: "Phone" },
			{ title: "Email" },
			{ title: "Address 1" },
			{ title: "Address 2" },
			{ title: "Delivery Date" }
		];

		if ('mulch' === frConfig.kind()) {
			tableColumns.push({ title: "Neighborhood" });
			tableColumns.push({ title: "Spreading" });
			tableColumns.push({ title: "Bags" });
		}

		tableColumns = tableColumns.concat([
			{ title: "Special Instructions" },
			{ title: "Donations" },
			{ title: "Cash" },
			{ title: "Check" },
			{ title: "Check Numbers" },
			{ title: "Total Amount" },
			{ title: "IsValidated" },

		]);

		tableColumns.push({
			title: "Actions",
			"orderable": false,
			className: "all"
		});

		this.currentDataTable_ = jQuery('#orderListTable').DataTable({
			data: orderDataSet,
			paging: false,
			bInfo : false,
			columns: tableColumns
		});

		this.registerActionButtonHandlers();
	}

	////////////////////////////////////////////////////////////////////
	//
	genCsvFromCurrent() {
		if (!this.currentDataTable_) { throw new Error("Table isn't found"); }
		let csvFileData = [];

		const headerElm = this.currentDataTable_.table().header();
		let csvRow = []
		for (const th of jQuery(headerElm).find(`th`)) {
			if ('Actions'===th.innerText) { continue; }
			console.log();
			csvRow.push(th.innerText);
		};
		csvRow = ['OrderId', 'OrderOwner'].concat(csvRow);
		csvFileData.push(csvRow.join('|'));

		const data = this.currentDataTable_.data().toArray();

		data.forEach((row, _)=>{
			csvRow = [];
			row.forEach((column, _)=>{
				csvRow.push(column);
			});
			csvRow.splice(-1,1);
			csvFileData.push(csvRow.join('|'));
		});

		//console.log(`${JSON.stringify(csvFileData, null, '\t')}`);


		/* const flattened = Object.assign(
		   {},
		   ...function _flatten(o) {
		   return [].concat(...Object.keys(o)
		   .map(k =>
		   typeof o[k] === 'object' ?
		   _flatten(o[k]) :
		   ({[k]: o[k]})
		   ));
		   }(this.currentQueryResults_)); */
		//console.log(`${JSON.stringify(flattened, null, '\t')}`);
		return csvFileData;
	}

}

const reportViews: ReportViews = new ReportViews();

////////////////////////////////////////////////////////////////////
//
const genDeleteDlg = ()=>{
	// Check for enabling/disabling Delete From Button
	const doesDeleteBtnGetEnabled = (event: any)=>{
		if ('delete'===event.currentTarget.value) {
			(document.getElementById('deleteDlgBtn') as HTMLButtonElement).disabled = false;
		} else {
			(document.getElementById('deleteDlgBtn') as HTMLButtonElement).disabled = true;
		}
	};

	return(
        <div className="modal fade" id="deleteOrderDlg"
             tabIndex="-1" role="dialog" aria-labelledby="deleteOrderDlgTitle" aria-hidden="true">
            <div className="modal-dialog modal-dialog-centered" role="document">
                <div className="modal-content">
                    <div className="modal-header">
                        <h5 className="modal-title" id="deleteOrderDlgLongTitle">
                            Confirm Order Deletion
                        </h5>
                        <button type="button" className="close" data-bs-dismiss="modal" aria-label="Close">
                            <span aria-hidden="true">&times;</span>
                        </button>
                    </div>
                    <div className="modal-body">
                        <input type="text" className="form-control" id="confirmDeleteOrderInput"
                               placeholder="type delete to confirm"  autoComplete="fr-new-cust-info"
                               onInput={doesDeleteBtnGetEnabled} aria-describedby="confirmDeleteOrderHelp" />
                        <small id="confirmDeleteOrderHelp" className="form-text text-muted">
                            Enter "delete" to confirm order deletion.
                        </small>

                    </div>
                    <div className="modal-footer">
                        <button type="button" disabled className="btn btn-primary" id="deleteDlgBtn">
                            Delete Order
                        </button>
                        <button type="button" className="btn btn-secondary" data-bs-dismiss="modal">Cancel</button>
                    </div>
                </div>
            </div>
        </div>
	);
};

////////////////////////////////////////////////////////////////////
//
const showTheSelectedView = (frConfig: FundraiserConfig, isAdmin: boolean) => {

	const showView = ()=>{
		const userSelElm = document.getElementById(`${dlgIdRoot}UserSelection`);
		const viewSelElm = document.getElementById(`${dlgIdRoot}ViewSelection`);

		//Update the selected view label
		const selectedUser = userSelElm.options[userSelElm.selectedIndex].text;
		const selectedView = viewSelElm.options[viewSelElm.selectedIndex].text;
		const rvLabel = document.getElementById("reportViewLabel");
		console.log(`${selectedView}(${selectedUser})`);
		rvLabel.innerText = `${selectedView}(${selectedUser})`;

		const userIdOverride = (isAdmin?userSelElm.options[userSelElm.selectedIndex].value:undefined);

		reportViews.show(selectedView, frConfig, userIdOverride);
	};

	// Check to see if initialized
	if (!document.getElementById(`${dlgIdRoot}UserSelection`)) {
		const genOption = (label, val)=>{
			const option = document.createElement("option");
			option.text = label;
			if (val) {
				option.value = val;
			}
			return option;
		};

		auth.getUserIdAndGroups().then(([_, userGroups])=>{
			const userSelElm = document.getElementById(`${dlgIdRoot}UserSelection`);
			const viewSelElm = document.getElementById(`${dlgIdRoot}ViewSelection`);
			if (userGroups && userGroups.includes("FrAdmins")) {
				console.log("This user is an admin");
				document.getElementById(`${dlgIdRoot}UserSelectionCol`).style.display = "inline-block";
			} else {
				const fullName = frConfig.getUserNameFromId(auth.getCurrentUserId())
				document.getElementById(`${dlgIdRoot}UserSelectionCol`).style.display = "none";

				userSelElm.add(genOption(fullName, auth.getCurrentUserId()));
				userSelElm.selectedIndex = 0;

				viewSelElm.add(genOption('Default'));
				viewSelElm.add(genOption('Full'));
				viewSelElm.selectedIndex = 1;
			}

			showView();
		});
	} else {
		showView();
	}

};

////////////////////////////////////////////////////////////////////
//
const genReportSettingsDlg = ()=>{
	return(
        <div className="modal fade" id={dlgIdRoot}
             tabIndex="-1" role="dialog" aria-labelledby={dlgIdRoot + "Title"} aria-hidden="true">
            <div className="modal-dialog modal-dialog-centered" role="document">
                <div className="modal-content">
                    <div className="modal-header">
                        <h5 className="modal-title" id={dlgIdRoot + "LongTitle"}>
                            Switch report view settings
                        </h5>
                        <button type="button" className="close" data-bs-dismiss="modal" aria-label="Close">
                            <span aria-hidden="true">&times;</span>
                        </button>
                    </div>
                    <div className="modal-body">
						<div className="container-sm">
							<div className="row">
								<div className="col-sm">
									<div className="form-floating">
										<select className="form-control" id={dlgIdRoot+"ViewSelection"}/>
										<label htmlFor={dlgIdRoot+"ViewSelection"}>
											Select Report View
										</label>
									</div>
								</div>
								<div className="col-sm" id={dlgIdRoot+"UserSelectionCol"}>
									<div className="form-floating">
										<select className="form-control" id={dlgIdRoot+"UserSelection"}/>
										<label htmlFor={dlgIdRoot+"UserSelection"}>
											Select User
										</label>
									</div>
								</div>
							</div>
						</div>
                    </div>
                    <div className="modal-footer">
                        <button type="button" className="btn btn-primary" data-bs-dismiss="modal" id={dlgIdRoot + "OnSave"}>
                            Save
                        </button>
                        <button type="button" className="btn btn-secondary" data-bs-dismiss="modal">Cancel</button>
                    </div>
                </div>
            </div>
        </div>
	);
};

////////////////////////////////////////////////////////////////////
//
const genCardBody = (frConfig: FundraiserConfig)=>{
	const fullName = frConfig.getUserNameFromId(auth.getCurrentUserId())

	const onVewSettingsClick = ()=>{
		auth.getUserIdAndGroups().then(([_, userGroups])=>{
			console.log("Settings Clicked");

			const dlgElm = document.getElementById(dlgIdRoot);
			reportSettingsDlg = new bs.Modal(dlgElm, {
				backdrop: true,
				keyboard: true,
				focus: true
			});

			document.getElementById(dlgIdRoot+"OnSave").onclick = (event)=>{
				console.log("Clicked");
				showTheSelectedView(frConfig);
			}

			reportSettingsDlg.show();
		});
	};


	const onDownloadReportClick = ()=>{
		const csvData = reportViews.genCsvFromCurrent().join('\n');
		const hiddenElement = document.createElement('a');
		hiddenElement.href = 'data:text/plain;charset=utf-8,' + encodeURI(csvData);
		hiddenElement.target = '_blank';
		hiddenElement.download = 'FundraisingReport.text';
		hiddenElement.click();
	};


	return(
        <div className="card-body" style={{padding: 0}}>
            <h5 className="card-title ps-2" id="orderCardTitle">
				Reports View: <div style={{display: "inline"}} id="reportViewLabel">Default({fullName})</div>
				<button type="button" className="btn reports-view-setting-btn" onClick={onVewSettingsClick}>
					<svg className="bi" fill="currentColor">
						<use xlinkHref={reportSettingsImg}/>
					</svg>
				</button>
				<button type="button" className="btn reports-view-setting-btn float-end" onClick={onDownloadReportClick}>
					<svg className="bi" fill="currentColor">
						<use xlinkHref={exportImg}/>
					</svg>
				</button>

			</h5>
            <table id="orderListTable"
                   className="display responsive nowrap table table-hover"
                   style={{width:"100%"}}/>
            <div className="spinner-border" role="status" id="orderLoadingSpinner">
                <span className="visually-hidden">Loading...</span>
            </div>
        </div>
	);
};


////////////////////////////////////////////////////////////////////
//
export default function orders() {

    const addNewOrder=()=>{
        console.log("Add new order");
        orderDb.newActiveOrder();
        navigate('/order_step_1/');
    };

    // Client-side Runtime Data Fetching
	const [cardBody, setCardBody] = useState();
	const [deleteDlg, setDeleteDlg] = useState();
	const [settingsDlg, setReportSettingsDlg] = useState();
    useEffect(() => {
		const frConfig = getFundraiserConfig();
		setCardBody(genCardBody(frConfig));
		setDeleteDlg(genDeleteDlg());
		setReportSettingsDlg(genReportSettingsDlg());

		showTheSelectedView(frConfig);

    }, []);


    return (
        <div>
            <NavBar/>

			<button type="button"
                    className="btn btn-outline-primary add-order-btn"
                    onClick={addNewOrder}>
				<svg className="bi" fill="currentColor">
					<use xlinkHref={addOrderImg}/>
				</svg>
            </button>

            <div className="col-xs-1 d-flex justify-content-center">
                <div className="card" >
					{cardBody}
                </div>
            </div>

			{deleteDlg}
			{settingsDlg}

        </div>
    );
}
