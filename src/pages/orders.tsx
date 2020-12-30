import React, { useState, useEffect } from "react";
import ReactDOM from 'react-dom'
import AddNewOrderWidget from "../components/add_new_order_widget"
import { navigate } from "gatsby";
import {orderDb, OrderListItem} from "../js/ordersdb";
import currency from "currency.js";
import auth from "../js/auth"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config";
import bootstrapIconSprite from "bootstrap-icons/bootstrap-icons.svg";
import jQuery from 'jquery'

const trashImg = bootstrapIconSprite + "#trash";
const pencilImg = bootstrapIconSprite + "#pencil";
const eyeImg = bootstrapIconSprite + "#eye";
const exportImg = bootstrapIconSprite + "#cloud-download";
const reportSettingsImg = bootstrapIconSprite + "#gear";
const spreadImg = bootstrapIconSprite + "#layout-wtf";

const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });
const rprtStngDlgRt = 'reportViewSettingsDlg';
const spreadingDlgRt = 'spreadingDlg';
let reportSettingsDlg = undefined;

////////////////////////////////////////////////////////////////////
//
class ReportViews {
    private currentView_: string = "";
    private currentUserId_: string = "";
    private currentDataTable_: any = undefined;
    private currentQueryResults_: Array<OrderListItem<string>> = undefined;

	/*
    constructor() {
        console.log("Constructing...");
		window.addEventListener('resize', ()=>{
			console.log(`Evt Screen Height ${screen.height} window innerHeight: ${window.innerHeight}`);
			if (this.currentDataTable_) {
				if (window.innerHeight
				   this.currentDataTable_.page.len(10).draw();
			}
		});
    }
	*/

    ////////////////////////////////////////////////////////////////////
    //
    showView(view: string, frConfig: FundraiserConfig, userId?: string) {
        const asyncShow = async () => {

            if (jQuery.fn.dataTable.isDataTable('#orderListTable table')) {
                if (view === this.currentView_ && userId === this.currentUserId_) { return; }

                jQuery('#orderListTable table').DataTable().clear();
                jQuery('#orderListTable table').DataTable().destroy();
                jQuery('#orderListTable table').empty();
                delete this.currentDataTable_;
                delete this.currentQueryResults_;
            }

            console.log(`Current View: ${this.currentView_} New View: ${view}`);
            console.log(`Current User: ${this.currentUserId_} New User: ${userId}`);
            this.currentView_ = view;
            this.currentUserId_ = userId;

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
                    throw err;
                }
            });
    }


    ////////////////////////////////////////////////////////////////////
    //
    private getActionButtons(order: any, frConfig: FundraiserConfig) {

        let htmlStr = `<div style="float: right">`;

        if (('mulch' === frConfig.kind()) && order.products?.spreading) {
            htmlStr +=
                `<button type="button" class="btn btn-outline-info me-1 order-spread-btn">` +
                `<svg class="bi" fill="currentColor"><use xlink:href="${spreadImg}" /></svg></button>`;
        }

        if (frConfig.isEditableDeliveryDate(order.deliveryId)) {
            htmlStr +=
                `<button type="button" class="btn btn-outline-info me-1 order-edt-btn">` +
                `<svg class="bi" fill="currentColor"><use xlink:href="${pencilImg}" /></svg></button>` +
                `<button type="button" class="btn btn-outline-danger order-del-btn">` +
                `<svg class="bi" fill="currentColor"><use xlink:href="${trashImg}" /></svg></button>`;
        } else {
            htmlStr +=
                `<button type="button" class="btn btn-outline-info me-1 order-view-btn">` +
                `<svg class="bi" fill="currentColor"><use xlink:href="${eyeImg}" /></svg></button>`;
        }

        htmlStr += `</div>`;
        return htmlStr;
    }

    ////////////////////////////////////////////////////////////////////
    //
    private genDataTable(orderDataSet: any, tableColumns: any) {
		//console.log(`Screen Height ${screen.height} window innerHeight: ${window.innerHeight}`);
		return jQuery('#orderListTable table').DataTable({
			responsive: true,
			data: orderDataSet,
			deferRender: true,
			drawCallback: ( settings: any ) => {
				// console.log("Draw Callback Called");
				this.registerActionButtonHandlers();
			},
			language: {
				paginate: {
					previous: "<<",
					next: ">>"
				}
			},
			columns: tableColumns
		});

	}


    ////////////////////////////////////////////////////////////////////
    //
    private registerActionButtonHandlers() {
		//Removing first so it doesn't get doubled loaded
		jQuery('#orderListTable').find('.order-edt-btn').off('click');
		jQuery('#orderListTable').find('.order-view-btn').off('click');
		jQuery('#orderListTable').find('.order-spread-btn').off('click');
		jQuery('#orderListTable').find('.order-del-btn').off('click');

		// Handle on Edit Scenario
		jQuery('#orderListTable').find('.order-edt-btn').on('click', (event: any)=>{
			const parentTr = jQuery(event.currentTarget).parents('tr');
            const row = this.currentDataTable_.row(parentTr);
			const rowData = row.data();
            const orderId = rowData[0];
            const orderOwner = rowData[rowData.length - 2];

            console.log(`Editing order for ${orderId}`);
            orderDb.setActiveOrder(); // Reset active order to let order edit for set it
            navigate('/order_step_1/', {state: {
                editOrderId: orderId,
                editOrderOwner: orderOwner
            }});
        });

        // Handle on View Scenario
        jQuery('#orderListTable').find('.order-view-btn').on('click', (event: any)=>{
            const parentTr = jQuery(event.currentTarget).parents('tr');
            const row = this.currentDataTable_.row(parentTr);
			const rowData = row.data();
            const orderId = rowData[0];
            const orderOwner = rowData[rowData.length - 2];

            console.log(`View order for ${orderId}`);
            orderDb.setActiveOrder(); // Reset active order to let order edit for set it
            navigate('/order_step_1/', {state: {
				editOrderId: orderId,
                editOrderOwner: orderOwner,
				isOrderReadOnly: true
			}});
        });

        // Handle on View Scenario
        jQuery('#orderListTable').find('.order-spread-btn').on('click', (event: any)=>{
            const parentTr = jQuery(event.currentTarget).parents('tr');
            const row = this.currentDataTable_.row(parentTr);
			const rowData = row.data();
            const orderId = rowData[0];
            const orderOwner = rowData[rowData.length - 2];

            console.log(`Spreading Dlg order for ${orderId}`);
            const dlgElm = document.getElementById(spreadingDlgRt);
            const spreadOrderDlg = new bootstrap.Modal(dlgElm, {
                backdrop: true,
                keyboard: true,
                focus: true
            });
            spreadOrderDlg.show();
        });

        // Handle On Delete Scenario
        jQuery('#orderListTable').find('.order-del-btn').on('click', (event: any)=>{
            const parentTr = jQuery(event.currentTarget).parents('tr');
            const row = this.currentDataTable_.row(parentTr);
			const rowData = row.data();
            const orderId = rowData[0];
            const orderOwner = rowData[rowData.length - 2];

            console.log(`Deleting order for ${orderId}`);
            jQuery('#confirmDeleteOrderInput').val('');
            parentTr.find('button').attr("disabled", true);

            const dlgElm = document.getElementById('deleteOrderDlg');
            const delOrderDlg = new bootstrap.Modal(dlgElm, {
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
                        orderDb.deleteOrder(orderId, orderOwner).then(()=>{
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
    private async showDefault(frConfig: FundraiserConfig, userId?: string) {

        const currentUserId =  auth.getCurrentUserId();
        if (!userId) { userId = currentUserId; }
        // Build query fields
        const fieldNames = ["orderId", "firstName", "lastName"];
        fieldNames.push("deliveryId");
        if ('mulch' === frConfig.kind()) {
            fieldNames.push("products.spreading");
        }

        if ('any'===userId) {
            fieldNames.push("orderOwner");
        }


        this.currentQueryResults_ = await orderDb.query({fields: fieldNames, orderOwner: userId});
        const orders = this.currentQueryResults_;
        //console.log(`Default Orders Page: ${JSON.stringify(orders)}`);


		/* ReactDOM.render(
		   <button
		   onClick={() => this.props.getDelCartItem({ rowID: rowData.RowID, code:
		   rowData.ProdCode })}
		   data-toggle='tooltip' data-placement='right' title='Delete Item From Cart'
		   className='btn btn-sm btn-danger'>
		   <i className="fas fa-times fa-lg"></i>
		   </button>
		 *     , td); */

        // Fill out rows of data
        const orderDataSet = [];
        for (const order of orders) {
            const nameStr = `${order.firstName}, ${order.lastName}`;
            const orderOwner = ('any'===userId)?order.orderOwner:userId;
            const orderDataItem = [order.orderId, nameStr];
            //only reason to not have a delivery date is if it is a donation
            const deliveryDate = order.deliveryId?frConfig.deliveryDateFromId(order.deliveryId):'donation';
            orderDataItem.push(deliveryDate);

            if ('mulch' === frConfig.kind()) {
                orderDataItem.push((order.products?.spreading?order.products.spreading:''));
            }
            orderDataItem.push(orderOwner);
            orderDataItem.push(this.getActionButtons(order, frConfig));
            orderDataSet.push(orderDataItem);
        }


        const tableColumns = [
            {
                title: "OrderId",
                className: "all",
                visible: false
            },
            {
                title: "Name",
                className: "all"
            },
            {
                title: "Delivery Date",
            }
        ];

        if ('mulch' === frConfig.kind()) {
            tableColumns.push({ title: "Spreading" });
        }

        tableColumns.push({
            title: "Order Owner",
            visible: ('any'===userId || userId !== currentUserId),
            render: (data)=>{
                //console.log(`Data: JSON.stringify(data)`);
                return frConfig.getUserNameFromId(data);
            }
        });

        tableColumns.push({
            title: "Actions",
            "orderable": false,
            className: "all"
        });

        this.currentDataTable_ = this.genDataTable(orderDataSet, tableColumns);

        //this.registerActionButtonHandlers();
    }

    ////////////////////////////////////////////////////////////////////
    //
    private async showVerifyOrders(frConfig: FundraiserConfig, userId?: string) {

        const currentUserId =  auth.getCurrentUserId();
        if (!userId) { userId = currentUserId; }
        // Build query fields
        const fieldNames = ["orderId", "firstName", "lastName", "deliveryId",
                            "totalAmt", "cashPaid", "checkPaid", "checkNums", "isVerified"];


        if ('any'===userId) {
            fieldNames.push("orderOwner");
        }


        this.currentQueryResults_ = await orderDb.query({fields: fieldNames, orderOwner: userId});
        const orders = this.currentQueryResults_;
        //console.log(`Verify Orders Page: ${JSON.stringify(orders)}`);

        const htmlValidateSwitch =
            `<div class="form-check form-switch">` +
            `<input class="form-check-input" type="checkbox" />` +
            `</div>`;

        // Fill out rows of data
        const orderDataSet = [];
        for (const order of orders) {
            const nameStr = `${order.firstName}, ${order.lastName}`;
            const orderOwner = ('any'===userId)?order.orderOwner:userId;
            //only reason to not have a delivery date is if it is a donation
            const deliveryDate = order.deliveryId?frConfig.deliveryDateFromId(order.deliveryId):'donation';
            const orderDataItem = [order.orderId, nameStr, deliveryDate, order.totalAmt,
                                   order.cashPaid?order.cashPaid:0,
                                   order.checkPaid?order.checkPaid:0,
                                   order.checkNums?order.checkNums:'',
                                   htmlValidateSwitch, orderOwner, this.getActionButtons(order, frConfig)];
            orderDataSet.push(orderDataItem);
        }

        const tableColumns = [
            { title: "OrderId", className: "all", visible: false },
            { title: "Name", className: "all" },
            { title: "Delivery Date" },
            { title: "Total Amt", render: (data)=>{ return USD(data).format(); } },
            { title: "Cash Paid", render: (data)=>{ return USD(data).format(); } },
            { title: "Checks Paid", render: (data)=>{ return USD(data).format(); } },
            { title: "Checks", render: (data)=>{ return(null!==data ? data : ''); } },
            {
                title: "Verify",
                className: "all"
            },
            {
                title: "Order Owner",
                visible: ('any'===userId || userId !== currentUserId),
                render: (data)=>{
                    //console.log(`Data: JSON.stringify(data)`);
                    return frConfig.getUserNameFromId(data);
                }
            },
            {
                title: "Actions",
                "orderable": false,
                className: "all"
            }
        ];

        this.currentDataTable_ = this.genDataTable(orderDataSet, tableColumns);

        //this.registerActionButtonHandlers();
    }

    ////////////////////////////////////////////////////////////////////
    //
    private async showFull(frConfig: FundraiserConfig, userId?: string) {

        const currentUserId =  auth.getCurrentUserId();
        if (!userId) { userId = currentUserId; }

        this.currentQueryResults_ = await orderDb.query({orderOwner: userId});
        const orders = this.currentQueryResults_;

        //console.log(`Full Orders Page: ${JSON.stringify(orders)}`);

        const getVal = (fld?: any, dflt?: any)=>{
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
            const orderOwner = ('any'===userId)?order.orderOwner:userId;
            const deliveryDate = order.deliveryId?frConfig.deliveryDateFromId(order.deliveryId):'donation';

            let orderDataItem = [
                order.orderId,
                nameStr,
                order.phone,
                getVal(order.email),
                order.addr1,
                getVal(order.addr2),
                deliveryDate
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
				true===order.isVerified?"Yes":"No",
				true===order.doCollectMoneyLater?'No':'Yes',
                USD(order.donation).format(),
                USD(order.cashPaid).format(),
                USD(order.checkPaid).format(),
                getVal(order.checkNums),
                USD(order.totalAmt).format(),
                (order.isVerified?"True":"False"),
                orderOwner,
                this.getActionButtons(order, frConfig)
            ]);
            orderDataSet.push(orderDataItem);
        }


        let tableColumns = [
            { title: "OrderId", className: "all", visible: false },
            { title: "Name", className: "all" },
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
            { title: "Verified" },
            { title: "Money Collected" },
            { title: "Donations" },
            { title: "Cash" },
            { title: "Check" },
            { title: "Check Numbers" },
            { title: "Total Amount" },
            { title: "IsValidated" },

        ]);

        tableColumns.push({
            title: "Order Owner",
            visible: ('any'===userId || userId !== currentUserId),
            render: (data)=>{
                //console.log(`Data: JSON.stringify(data)`);
                return frConfig.getUserNameFromId(data);
            }
        });

        tableColumns.push({
            title: "Actions",
            "orderable": false,
            className: "all"
        });

        this.currentDataTable_ = this.genDataTable(orderDataSet, tableColumns);

        //this.registerActionButtonHandlers();
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
        return csvFileData;
    }

}

let reportViews: ReportViews = new ReportViews();

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
                        <button type="button" className="btn btn-secondary"
                                data-bs-dismiss="modal">Cancel</button>
                    </div>
                </div>
            </div>
        </div>
    );
};


////////////////////////////////////////////////////////////////////
//
const showTheSelectedView = async (frConfig: FundraiserConfig) => {
    const userSelElm = document.getElementById(`${rprtStngDlgRt}UserSelection`);
    const viewSelElm = document.getElementById(`${rprtStngDlgRt}ViewSelection`);

    const showView = ()=>{
        //Update the selected view label
        const selectedUser = userSelElm.options[userSelElm.selectedIndex].text;
        const selectedViewOpt = viewSelElm.options[viewSelElm.selectedIndex];
        const rvLabel = document.getElementById("reportViewLabel");
        const rvOrderOwner = document.getElementById("reportViewOrderOwner");
        console.log(`${selectedViewOpt.text}(${selectedUser})`);
        rvLabel.innerText = `${selectedViewOpt.text}`;
        rvOrderOwner.innerText = `${selectedUser}`;

        const userIdOverride = userSelElm.options[userSelElm.selectedIndex].value;
        reportViews.showView(selectedViewOpt.value, frConfig, userIdOverride);
    };

    // Check to see if Report Views User view has been initialized
    if (0===userSelElm.options.length) {
        const genOption = (label, val)=>{
            const option = document.createElement("option");
            option.text = label;
            option.value = val ? val : label;
            return option;
        };

        const [_, userGroups] = auth.getUserIdAndGroups();
        const userSelElm = document.getElementById(`${rprtStngDlgRt}UserSelection`);
        const viewSelElm = document.getElementById(`${rprtStngDlgRt}ViewSelection`);

        const fullName = frConfig.getUserNameFromId(auth.getCurrentUserId())

        if (userGroups && userGroups.includes("FrAdmins")) {
            for (const userInfo of frConfig.users()) {
                userSelElm.add(genOption(userInfo[1], userInfo[0]));
            }
            userSelElm.add(genOption('All', 'any'));
            userSelElm.value = auth.getCurrentUserId();
            document.getElementById(`${rprtStngDlgRt}UserSelectionCol`).style.display = "inline-block";

            viewSelElm.add(genOption('Default'));
            viewSelElm.add(genOption('Full'));
            viewSelElm.add(genOption('Verify Orders', 'VerifyOrders'));
            viewSelElm.selectedIndex = 0;
        } else {
            document.getElementById(`${rprtStngDlgRt}UserSelectionCol`).style.display = "none";
            userSelElm.add(genOption(fullName, auth.getCurrentUserId()));
            userSelElm.selectedIndex = 0;

            viewSelElm.add(genOption('Default'));
            viewSelElm.add(genOption('Full'));
            viewSelElm.selectedIndex = 0;
        }

        showView();
		console.log("Show View Not Initted");
    } else {
		console.log("Show View Initted");
        showView();
    }

};

////////////////////////////////////////////////////////////////////
//
const genReportSettingsDlg = ()=>{
    return(
        <div className="modal fade" id={rprtStngDlgRt}
             tabIndex="-1" role="dialog" aria-labelledby={rprtStngDlgRt + "Title"} aria-hidden="true">
            <div className="modal-dialog modal-dialog-centered" role="document">
                <div className="modal-content">
                    <div className="modal-header">
                        <h5 className="modal-title" id={rprtStngDlgRt + "LongTitle"}>
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
                                        <select className="form-control" id={rprtStngDlgRt+"ViewSelection"}/>
                                        <label htmlFor={rprtStngDlgRt+"ViewSelection"}>
                                            Select Report View
                                        </label>
                                    </div>
                                </div>
                                <div className="col-sm" id={rprtStngDlgRt+"UserSelectionCol"}>
                                    <div className="form-floating">
                                        <select className="form-control" id={rprtStngDlgRt+"UserSelection"} />
                                        <label htmlFor={rprtStngDlgRt+"UserSelection"}>
                                            Select User
                                        </label>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div className="modal-footer">
                        <button type="button" className="btn btn-primary"
                                data-bs-dismiss="modal" id={rprtStngDlgRt + "OnSave"}>
                            Save
                        </button>
                        <button type="button" className="btn btn-secondary"
                                data-bs-dismiss="modal">Cancel</button>
                    </div>
                </div>
            </div>
        </div>
    );
};

////////////////////////////////////////////////////////////////////
//
const genSpreadingDlg = () => {
    return(
        <div className="modal fade" id={spreadingDlgRt}
             tabIndex="-1" role="dialog" aria-labelledby={spreadingDlgRt + "Title"} aria-hidden="true">
            <div className="modal-dialog modal-dialog-centered" role="document">
                <div className="modal-content">
                    <div className="modal-header">
                        <h5 className="modal-title" id={spreadingDlgRt + "LongTitle"}>
                            Spreading Completion
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
                                        <div className="form-check form-switch">
                                            <input className="form-check-input" type="checkbox" id="isSpreadCheck" />
                                            <label className="form-check-label"
                                                   htmlFor="isSpreadCheck">Spreading Complete</label>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div className="modal-footer">
                        <button type="button" className="btn btn-primary"
                                data-bs-dismiss="modal" id={spreadingDlgRt + "OnSave"}>
                            Save
                        </button>
                        <button type="button" className="btn btn-secondary"
                                data-bs-dismiss="modal">Cancel</button>
                    </div>
                </div>
            </div>
        </div>
    );
};

////////////////////////////////////////////////////////////////////
//
const genCardBody = (frConfig: FundraiserConfig) => {
    const fullName = frConfig.getUserNameFromId(auth.getCurrentUserId());

    const onVewSettingsClick = ()=>{
        const dlgElm = document.getElementById(rprtStngDlgRt);
        reportSettingsDlg = new bootstrap.Modal(dlgElm, {
            backdrop: true,
            keyboard: true,
            focus: true
        });

        document.getElementById(rprtStngDlgRt+"OnSave").onclick = (event)=>{
            showTheSelectedView(frConfig);
        }

        reportSettingsDlg.show();
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
        <div className="card-body" id="cardReportBody">
            <h6 className="card-title ps-2" id="orderCardTitle">
                <ul className="list-group list-group-horizontal-sm">
                    <li className="list-group-item me-2">
                        Report View:<div className="d-inline" id="reportViewLabel">Default</div>
                    </li>
                    <li className="list-group-item" id="orderOwnerLabel">
                        Order Owner:<div className="d-inline" id="reportViewOrderOwner">{fullName}</div>
                    </li>
                </ul>
                <div id="reportViewSettings" className="float-end">
                    <button type="button" className="btn reports-view-setting-btn" onClick={onDownloadReportClick}>
                        <svg className="bi" fill="currentColor">
                            <use xlinkHref={exportImg}/>
                        </svg>
                    </button>
                    <button type="button" className="btn reports-view-setting-btn" onClick={onVewSettingsClick}>
                        <svg className="bi" fill="currentColor">
                            <use xlinkHref={reportSettingsImg}/>
                        </svg>
                    </button>
                </div>
            </h6>

			<div id="orderListTable">
				<table  className="display responsive nowrap collapsed" role="grid" style={{width:"100%"}}/>
			</div>

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
    const [spreadDlg, setSpreadDlg] = useState();
    const [settingsDlg, setReportSettingsDlg] = useState();
    useEffect(() => {
		const onLoadComponent = async ()=>{
			const [isValidSession, session] = await auth.getSession();
            if (!isValidSession) {
                // If no active user go to login screen
                navigate('/signon/');
                return;
            }

			const frConfig = getFundraiserConfig();

			console.log("loaded FrConfig");

			console.log("Loading Gen Card Body");
			setCardBody(genCardBody(frConfig));
			setDeleteDlg(genDeleteDlg());
			setSpreadDlg(genSpreadingDlg());
			setReportSettingsDlg(genReportSettingsDlg());


			await showTheSelectedView(frConfig);
			const [_, userGroups] = await auth.getUserIdAndGroups();
			const isAdmin = (userGroups && userGroups.includes("FrAdmins"));
			if (!isAdmin) { document.getElementById("orderOwnerLabel").style.display = "none"; }
		};

		onLoadComponent()
			.then(()=>{})
			.catch((err)=>{
				if ('Invalid Session'===err.message) {
					navigate('/signon/');
					return;
				} else {
					console.error(err);
				}
			});

    }, []);


    return (
        <div>
            <div className="col-xs-1 d-flex justify-content-center">
                <div className="card" style={{width: "100%"}}>
                    {cardBody}
                </div>
            </div>

			<AddNewOrderWidget/>

            {deleteDlg}
            {settingsDlg}
            {spreadDlg}

        </div>
    );
}
