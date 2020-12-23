import React, { useState, useEffect } from "react";
import NavBar from "../components/navbar";
import { navigate } from "gatsby";
import {orderDb, OrderListItem} from "../js/ordersdb";
import currency from "currency.js";
import jQuery from 'jquery';
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config";
import bootstrapIconSprite from "bootstrap-icons/bootstrap-icons.svg";

const addOrderImg = bootstrapIconSprite + "#plus-square-fill";
const trashImg = bootstrapIconSprite + "#trash";
const pencilImg = bootstrapIconSprite + "#pencil";


export default function orders() {
    const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });

    const addNewOrder=()=>{
        console.log("Add new order");
        orderDb.newActiveOrder();
        navigate('/order_step_1/');
    };

    // Client-side Runtime Data Fetching
    useEffect(() => {
		const frConfig = getFundraiserConfig();

		// Build query fields
		const fieldNames = ["orderId", "firstName", "lastName"];
		if ('mulch' === frConfig.kind()) {
			fieldNames.push("deliveryId");
			fieldNames.push("products.spreading");
		}

        orderDb.query({fields: fieldNames}).then((orders: Array<OrderListItem<string>>)=>{
            console.log(`Orders Page: ${JSON.stringify(orders)}`);

			// Fill out rows of data
            const orderDataSet = [];
            for (const order of orders) {
                const nameStr = `${order.firstName}, ${order.lastName}`;

				const orderDataItem = [order.orderId, nameStr];
				if ('mulch' === frConfig.kind()) {
					if (order.deliveryId) {
						orderDataItem.push(frConfig.deliveryDateFromId(order.deliveryId));
					} else {
						orderDataItem.push('');
					}
					orderDataItem.push((order.products?.spreading?"Yes":"No"));
				}
				orderDataItem.push(null);
                orderDataSet.push(orderDataItem);
            }

            if (jQuery.fn.dataTable.isDataTable( '#orderListTable')) {
                jQuery('#orderListTable').DataTable().destroy();
            }

            const buttonDef =
                `<div>` +
                `<button type="button" class="btn btn-outline-info me-1 order-edt-btn">` +
				`<span><svg class="bi" fill="currentColor"><use xlink:href=${pencilImg}/></svg></span></button>` +
                `<button type="button" class="btn btn-outline-danger order-edt-btn">` +
				`<span><svg class="bi" fill="currentColor"><use xlink:href=${trashImg}/></svg></span></button>` +
                `</div>`;

			const tableColumns = [
                {
                    title: "OrderID",
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
                data: null,
                "orderable": false,
                "defaultContent": buttonDef,
                className: "all"
            });

            const table = jQuery('#orderListTable').DataTable({
                data: orderDataSet,
                paging: false,
                bInfo : false,
                columns: tableColumns
            });

            // Handle on Edit Scenario
            jQuery('#orderListTable').find('.btn-outline-info').on('click', (event: any)=>{
                const parentTr = jQuery(event.currentTarget).parents('tr');
                const row = table.row(parentTr);
                const orderId = row.data()[0];

                console.log(`Editing order for ${orderId}`);
                orderDb.setActiveOrder(); // Reset active order to let order edit for set it
                navigate('/order_step_1/', {state: {editOrderId: orderId}});
            });

            // Handle On Delete Scenario
            jQuery('#orderListTable').find('.btn-outline-danger').on('click', (event: any)=>{
                const parentTr = jQuery(event.currentTarget).parents('tr');
                const row = table.row(parentTr);
                const orderId = row.data()[0];

                console.log(`Deleting order for ${orderId}`);
                jQuery('#confirmDeleteOrderInput').val('');
                parentTr.find('button').attr("disabled", true);

				const dlgElm = document.getElementById('deleteOrderDlg');
				const delOrderDlg = new Modal(dlgElm, {
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
            } );

            const spinnerElm = document.getElementById('orderLoadingSpinner');
            if (spinnerElm) {
                spinnerElm.className = "d-none";
            }

        }).catch((err: any)=>{
            if ('Invalid Session'===err) {
                navigate('/signon/')
            } else {
                const errStr = `Failed creating order list: ${JSON.stringify(err)}`;
                console.log(errStr);
                alert(errStr);
                throw err;
            }
        });
    }, [])

    // Check for enabling/disabling Delete From Button
    const doesDeleteBtnGetEnabled = (event: any)=>{
        if ('delete'===event.currentTarget.value) {
            (document.getElementById('deleteDlgBtn') as HTMLButtonElement).disabled = false;
        } else {
            (document.getElementById('deleteDlgBtn') as HTMLButtonElement).disabled = true;
        }
    };


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
                <div className="card" style={{width: '80rem'}} >
                    <div className="card-body">
                        <h5 className="card-title" id="orderCardTitle">Orders</h5>
                        <table id="orderListTable"
                               className="display responsive nowrap table table-striped table-bordered table-hover"
                               style={{width:"100%"}}/>
                        <div className="spinner-border" role="status" id="orderLoadingSpinner">
                            <span className="visually-hidden">Loading...</span>
                        </div>
                    </div>
                </div>
            </div>

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

        </div>
    );
}
