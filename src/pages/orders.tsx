import React, { useState, useEffect } from "react";
import ReactDOM from 'react-dom'
import AddNewOrderWidget from "../components/add_new_order_widget"
import { navigate } from "gatsby";
import {orderDb, OrderListItem} from "../js/ordersdb";
import currency from "currency.js";
import auth from "../js/auth"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config";
import bootstrapIconSprite from "bootstrap-icons/bootstrap-icons.svg";

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
let lastAuthenticatedUser = undefined;

////////////////////////////////////////////////////////////////////
//
const resetSpreaderDlg = ()=>{
    // reset dlg to default state.  I.E. have selection btn be active view
    // and reset hidden currentOrderId, currentOrderOwner
    const reviewSelections = document.getElementById(spreadingDlgRt+"SpreaderSelectionReview");
    document.getElementById('spreadingSubmitBtnSpinny').style.display = 'none';
    for (let i = reviewSelections.options.length-1; i >= 0; i--) {
        reviewSelections.remove(i);
    }
    const selectedOpts = document.getElementById(`${spreadingDlgRt}SpreaderSelection`).options;
    for (const anOpt of selectedOpts) {
        anOpt.selected=false;
    }
    document.getElementById('spreadSelectTab').style.display='block';
    document.getElementById('spreadReviewTab').style.display='none';
    
    document.getElementById("spreadersSelectBtn").checked = true;
    document.getElementById("spreadersSaveBtn").classList.add("make-disabled");
};


////////////////////////////////////////////////////////////////////
//
class ReportViews {
    private currentView_: string;
    private currentUserId_: string;
    private currentDataTable_: any = undefined;
    private currentQueryResults_: Array<OrderListItem<string>> = undefined;


    constructor() {
        /* console.log("Constructing...");
         * window.addEventListener('resize', ()=>{
         *     console.log(`Evt Screen Height ${screen.height} window innerHeight: ${window.innerHeight}`);
         *     if (this.currentDataTable_) {
         *         if (window.innerHeight
         *             this.currentDataTable_.page.len(10).draw();
         *     }
         *         }); */
    }


    ////////////////////////////////////////////////////////////////////
    //
    async getViewOptions() {
        const [_, userGroups] = await auth.getUserIdAndGroups();
        const frConfig = getFundraiserConfig();
        const views=[];
        const users=[];

        if (userGroups && userGroups.includes("FrAdmins")) {
            views.push(['Default', undefined]);
            if ('mulch' === frConfig.kind()) {
                views.push(['Spreading Jobs', 'SpreadingJobs']);
            }
            views.push(['Full', undefined]);
            views.push(['Verify Orders', 'VerifyOrders']);

            for (const userInfo of frConfig.users()) {
                users.push(userInfo);
            }
            users.push(['any', 'All']);
        } else {
            views.push(['Default', undefined]);
            if ('mulch' === frConfig.kind()) {
                views.push(['Spreading Jobs', 'SpreadingJobs']);
            }
            views.push(['Full', undefined]);

            const fullName = frConfig.getUserNameFromId(auth.getCurrentUserId())
            const uid = auth.getCurrentUserId();
            users.push([uid, fullName]);
        }
        return [views, users];
    }

    ////////////////////////////////////////////////////////////////////
    //
    resetToDefault() {
        this.currentUserId_ = auth.getCurrentUserId();
        this.currentView_ = 'Default';
    }

    ////////////////////////////////////////////////////////////////////
    //
    getCurrentUserId() { return this.currentUserId_; }

    ////////////////////////////////////////////////////////////////////
    //
    getCurrentView() { return this.currentView_; }

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
        jQuery('#orderListTable').find('.order-vrfy-switch').off('change');
        
        // Handle on Edit Scenario
        jQuery('#orderListTable').find('.order-edt-btn').on('click', (event: any)=>{
            const parentTr = jQuery(event.currentTarget).parents('tr');
            const row = this.currentDataTable_.row(parentTr);
            const rowData = row.data();
            const orderId = rowData[this.currentDataTable_.column('OrderId:name').index()];
            const orderOwner = rowData[this.currentDataTable_.column('OrderOwner:name').index()];

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
            const orderId = rowData[this.currentDataTable_.column('OrderId:name').index()];
            const orderOwner = rowData[this.currentDataTable_.column('OrderOwner:name').index()];

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
            const orderId = rowData[this.currentDataTable_.column('OrderId:name').index()];
            const orderOwner = rowData[this.currentDataTable_.column('OrderOwner:name').index()];
            const spreadersIdx = this.currentDataTable_.column('Spreaders:name').index();

            if (!(orderOwner && orderId)) {
                throw new Error("For spreading both orderOwner and orderID needs to be set to valid value");
            }

            // Reset so the form comes up pristine
            resetSpreaderDlg();
            // Now populate with existing spreader data.
            const selectedOpts = document.getElementById(`${spreadingDlgRt}SpreaderSelection`).options;
            for (const anOpt of selectedOpts) {
                for(const spreader of rowData[spreadersIdx]) {
                    if (anOpt.value === spreader) {
                        anOpt.selected = true;
                    }
                }
            }
                        
            // This works differently from delete dlg as most functionailty is in genDlg
            console.log(`Spreading Dlg order for ${orderId}:${orderOwner}`);
            const dlgElm = document.getElementById(spreadingDlgRt);
            const spreadOrderDlg = new bootstrap.Modal(dlgElm, {
                backdrop: true,
                keyboard: true,
                focus: true
            });

            jQuery('#spreadersSaveBtn')
                .off('click')
                .click((event: any)=>{
                    // record existing selections
                    if (!(orderOwner && orderId)) {
                        throw new Error("Trying to save without orderOwner or orderId");
                    }

                    const reviewSelections = document.getElementById(spreadingDlgRt+"SpreaderSelectionReview");

                    const spreaders=[];
                    for (const anOpt of reviewSelections.options) {
                        spreaders.push(anOpt.value);
                    }

                    if (0===spreaders.length) {
                        throw new Error("No spreaders selected so can't save");
                    }

                    // submit order update
                    console.log(`Submitting spreading job for ` +
                                `${orderId}:${orderOwner}: ${JSON.stringify(spreaders)}`);

                    // Start submit spnner
                    document.getElementById('spreadingSubmitBtnSpinny').style.display = "inline-block";

                    orderDb.submitSpreadingComplete({
                        orderOwner: orderOwner,
                        orderId: orderId,
                        spreaders: spreaders
                    }).then(()=>{
                        resetSpreaderDlg();
                        spreadOrderDlg.hide();
                        rowData[spreadersIdx] = spreaders;
                        row.data(rowData).draw();
                    }).catch((err:any)=>{
                        if ('Invalid Session'===err) {
                            navigate('/signon/')
                        } else {
                            submitSpinner.style.display = "none";
                            const errStr = `Failed submitting order: ${JSON.stringify(err)}`;
                            console.log(errStr);
                            alert(errStr);
                            throw err;
                        }
                    });
                    
                });
            
            spreadOrderDlg.show();
        });

        // Handle On Delete Scenario
        jQuery('#orderListTable').find('.order-del-btn').on('click', (event: any)=>{
            const parentTr = jQuery(event.currentTarget).parents('tr');
            const row = this.currentDataTable_.row(parentTr);
            const rowData = row.data();
            const orderId = rowData[this.currentDataTable_.column('OrderId:name').index()];
            const orderOwner = rowData[this.currentDataTable_.column('OrderOwner:name').index()];

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

        jQuery('#orderListTable').find('.order-vrfy-switch').on('change', (event: any)=>{
            const parentTr = jQuery(event.currentTarget).parents('tr');
            const row = this.currentDataTable_.row(parentTr);
            const rowData = row.data();
            const orderId = rowData[this.currentDataTable_.column('OrderId:name').index()];
            const orderOwner = rowData[this.currentDataTable_.column('OrderOwner:name').index()];
            const verifyIdx = this.currentDataTable_.column('Verify:name').index();
            console.log(`Verifiying order for ${orderId}`);

            event.preventDefault();
			      event.stopPropagation();

            // Don't allow change to happen here
            const newVal = event.currentTarget.checked;
            event.currentTarget.checked = !newVal;
            
            const dlgElm = document.getElementById('confirmDlg');
            const confirmDlg = new bootstrap.Modal(dlgElm, {
                backdrop: true,
                keyboard: true,
                focus: true
            });
            jQuery('#confirmDlgBtn')
                .off('click')
                .click((evt: any)=> {
                    event.currentTarget.checked = newVal;
                    confirmDlg.hide();

                    // Start submit spnner
                    const submitSpinner = document.getElementById('confirmDlgBtnSpinny');
                    submitSpinner.style.display = "inline-block";

                    orderDb.submitVerification({
                        orderOwner: orderOwner,
                        orderId: orderId,
                        isVerified: newVal
                    }).then(()=>{
                        submitSpinner.style.display = "none";
                        confirmDlg.hide();

                        console.log(`Saved Verifiying order for ${orderId} ${newVal}`);

                        rowData[verifyIdx] = newVal;
                        row.data(rowData).draw();
                    }).catch((err:any)=>{
                        if ('Invalid Session'===err) {
                            navigate('/signon/')
                        } else {
                            submitSpinner.style.display = "none";
                            const errStr = `Failed submitting order: ${JSON.stringify(err)}`;
                            console.log(errStr);
                            alert(errStr);
                            throw err;
                        }
                    });




                    
                });

            confirmDlg.show();
        });

    }
    ////////////////////////////////////////////////////////////////////
    //
    private async showDefault(frConfig: FundraiserConfig, userId?: string) {

        const currentUserId =  auth.getCurrentUserId();
        if (!userId) { userId = currentUserId; }
        // Build query fields
        const fieldNames = ["orderId", "firstName", "lastName", "deliveryId"];
        if ('mulch' === frConfig.kind()) {
            fieldNames.push("spreaders");
            fieldNames.push("products.spreading");
        }

        if ('any'===userId) {
            fieldNames.push("orderOwner");
        }


        this.currentQueryResults_ = await orderDb.query({fields: fieldNames, orderOwner: userId});
        const orders = this.currentQueryResults_;
        //console.log(`Default Orders Page: ${JSON.stringify(orders)}`);

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
                orderDataItem.push(order.spreaders?order.spreaders:[]);
                orderDataItem.push((order.products?.spreading?order.products.spreading:''));
            }
            orderDataItem.push(orderOwner);
            orderDataItem.push(this.getActionButtons(order, frConfig));
            orderDataSet.push(orderDataItem);
        }


        const tableColumns = [
            {
                name: "OrderId",
                className: "all",
                visible: false
            },
            {
                title: "Name",
                className: "all"
            },
            {
                title: "Delivery Date",
                name: "DeliveryDate"
            }
        ];

        if ('mulch' === frConfig.kind()) {
            tableColumns.push({
                title: "Spreaders",
                name: "Spreaders",
                visible: false
            });
            tableColumns.push({
                title: "Spreading",
                className: "all",
                render: (data, _, row, meta) => {
                    if (0!==row[meta.col-1].length) {
                        return `${data}: Spread`
                    } else {
                        return data;
                    }
                }
            });
        }

        tableColumns.push({
            title: "Order Owner",
            name: "OrderOwner",
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
    private async showSpreadingJobs(frConfig: FundraiserConfig, userId?: string) {

        const currentUserId =  auth.getCurrentUserId();
        if (!userId) { userId = currentUserId; }
        // Build query fields
        const fieldNames = ["orderId", "firstName", "lastName", "deliveryId", "addr1", "addr2",
                            "specialInstructions", "neighborhood"];
        if ('mulch' === frConfig.kind()) {
            fieldNames.push("spreaders");
            fieldNames.push("products.spreading");
        }

        if ('any'===userId) {
            fieldNames.push("orderOwner");
        }


        this.currentQueryResults_ = await orderDb.query({fields: fieldNames, orderOwner: userId});
        const orders = this.currentQueryResults_;
        //console.log(`Default Orders Page: ${JSON.stringify(orders)}`);

        // Fill out rows of data
        const orderDataSet = [];
        const nowDate = Date.now();
        for (const order of orders) {
            const deliveryDate = order.deliveryId?frConfig.deliveryDateFromId(order.deliveryId):'donation';
           
            if (!(order.products?.spreading && deliveryDate!=='donation' && nowDate > Date.parse(deliveryDate))) {
                continue;
            }
            const nameStr = `${order.firstName}, ${order.lastName}`;
            const orderOwner = ('any'===userId)?order.orderOwner:userId;
            const orderDataItem = [order.orderId, nameStr];
            //only reason to not have a delivery date is if it is a donation
            orderDataItem.push(deliveryDate);
            orderDataItem.push(`${order.specialInstructions?order.specialInstructions:''}`);
            orderDataItem.push(`${order.addr1} ${order.addr2?order.addr2:''}`);
            if ('mulch' === frConfig.kind()) {
                orderDataItem.push(order.neighborhood);
                orderDataItem.push(order.spreaders?order.spreaders:[]);
                orderDataItem.push((order.products?.spreading?order.products.spreading:''));
            }
            orderDataItem.push(orderOwner);
            orderDataItem.push(this.getActionButtons(order, frConfig));
            orderDataSet.push(orderDataItem);
        }


        const tableColumns = [
            {
                name: "OrderId",
                className: "all",
                visible: false
            },
            {
                title: "Name",
                className: "all"
            },
            {
                title: "Delivery Date",
                name: "DeliveryDate"
            },
            {
                title: "Instructions"
            },
            {
                title: "Address"
            }
        ];

        if ('mulch' === frConfig.kind()) {
            tableColumns.push({
                title: "Neighborhood",
                className: "all"
            });
            tableColumns.push({
                title: "Spreaders",
                name: "Spreaders",
                visible: false
            });
            tableColumns.push({
                title: "Spreading",
                className: "all",
                render: (data, _, row, meta) => {
                    if (0!==row[meta.col-1].length) {
                        return `${data}: Spread`
                    } else {
                        return data;
                    }
                }
            });
        }

        tableColumns.push({
            title: "Order Owner",
            name: "OrderOwner",
            visible: ('any'===userId || userId !== currentUserId),
            render: (data)=>{
                //console.log(`Data: JSON.stringify(data)`);
                return frConfig.getUserNameFromId(data);
            }
        });

        tableColumns.push({
            title: "Actions",
            "orderable": false
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

        /* const htmlValidateSwitch =
         *     `<div class="form-check form-switch">` +
         *     `<input class="form-check-input order-vrfy-switch" type="checkbox" />` +
         *     `</div>`;
         */
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
                                   orderOwner, order.isVerified?true:false,
                                   this.getActionButtons(order, frConfig)];
            orderDataSet.push(orderDataItem);
        }

        const tableColumns = [
            { name: "OrderId", className: "all", visible: false },
            { title: "Name", className: "all" },
            { title: "Delivery Date" },
            { title: "Total Amt", render: (data)=>{ return USD(data).format(); } },
            { title: "Cash Paid", render: (data)=>{ return USD(data).format(); } },
            { title: "Checks Paid", render: (data)=>{ return USD(data).format(); } },
            { title: "Checks", render: (data)=>{ return(null!==data ? data : ''); } },
            {
                title: "Order Owner",
                name: "OrderOwner",
                visible: ('any'===userId || userId !== currentUserId),
                render: (data)=>{
                    //console.log(`Data: JSON.stringify(data)`);
                    return frConfig.getUserNameFromId(data);
                }
            },
            {
                title: "Verify",
                name: "Verify",
                className: "all",
                render: (isVerified, renderType)=>{
                    if (renderType !== 'display') { return(isVerified); }
                    if (isVerified) {
                        return( `<div class="form-check form-switch">` +
                                `<input class="form-check-input order-vrfy-switch" type="checkbox" checked />` +
                                `</div>`);
                    } else {
                        return( `<div class="form-check form-switch">` +
                                `<input class="form-check-input order-vrfy-switch" type="checkbox" />` +
                                `</div>`);
                    }
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
                orderDataItem.push(order.spreaders?order.spreaders:[]);
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
            { name: "OrderId", className: "all", visible: false },
            { title: "Name", className: "all" },
            { title: "Phone" },
            { title: "Email" },
            { title: "Address 1" },
            { title: "Address 2" },
            { title: "Delivery Date" }
        ];

        if ('mulch' === frConfig.kind()) {
            tableColumns.push({ title: "Neighborhood" });
            tableColumns.push({
                title: "Spreaders",
                name: "Spreaders",
                visible: false
            });
            tableColumns.push({
                title: "Spreading",
                render: (data, _, row, meta) => {
                    if (0!==row[meta.col-1].length) {
                        return `${data}: Spread`
                    } else {
                        return data;
                    }
                }
            });
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
            name: "OrderOwner",
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
                    <button type="button" className="btn btn-secondary"
                            data-bs-dismiss="modal">Cancel</button>
                    <div className="modal-footer">
                        <button type="button" disabled className="btn btn-primary" id="deleteDlgBtn">
                            Delete Order
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
};


////////////////////////////////////////////////////////////////////
//
const showTheSelectedView = async (frConfig: FundraiserConfig) => {
    const genOption = (label, val)=>{
        const option = document.createElement("option");
        option.text = label;
        option.value = val ? val : label;
        return option;
    };

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
        const userSelElm = document.getElementById(`${rprtStngDlgRt}UserSelection`);
        const viewSelElm = document.getElementById(`${rprtStngDlgRt}ViewSelection`);

        const [views, users] = await reportViews.getViewOptions();
        for (const userInfo of users) {
            userSelElm.add(genOption(userInfo[1], userInfo[0]));
        }
        userSelElm.value = reportViews.getCurrentUserId();

        for (const reportView of views) {
            viewSelElm.add(genOption(reportView[0], reportView[1]));
        }
        viewSelElm.value = reportViews.getCurrentView();

        const [_, userGroups] = await auth.getUserIdAndGroups();
        if (userGroups && userGroups.includes("FrAdmins")) {
            document.getElementById(`${rprtStngDlgRt}UserSelectionCol`).style.display = "inline-block";
        } else {
            document.getElementById(`${rprtStngDlgRt}UserSelectionCol`).style.display = "none";
        }
        showView();
    } else {
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
                                        <select className="form-select" id={rprtStngDlgRt+"ViewSelection"}/>
                                        <label htmlFor={rprtStngDlgRt+"ViewSelection"}>
                                            Select Report View
                                        </label>
                                    </div>
                                </div>
                                <div className="col-sm" id={rprtStngDlgRt+"UserSelectionCol"}>
                                    <div className="form-floating">
                                        <select className="form-select" id={rprtStngDlgRt+"UserSelection"} />
                                        <label htmlFor={rprtStngDlgRt+"UserSelection"}>
                                            Select User
                                        </label>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div className="modal-footer">
                        <button type="button" className="btn btn-secondary"
                                data-bs-dismiss="modal">Cancel</button>
                        <button type="button" className="btn btn-primary"
                                data-bs-dismiss="modal" id={rprtStngDlgRt + "OnSave"}>
                            Save
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
};

////////////////////////////////////////////////////////////////////
//
const genSpreadingDlg = (frConfig: FundraiserConfig) => {

    /////////////////////////////////////
    //
    const handleSelection = (evt)=>{
        evt.preventDefault();
        evt.currentTarget.selected = !evt.currentTarget.selected;
        return false;
    };

    /////////////////////////////////////
    //
    const onSelect = ()=>{
        const spreaderSelectTabElm = document.getElementById('spreadSelectTab');
        spreaderSelectTabElm.style.display='block';
        const spreaderReviewTabElm = document.getElementById('spreadReviewTab');
        spreaderReviewTabElm.style.display='none';

        document.getElementById("spreadersSaveBtn").classList.add("make-disabled");
    };

    /////////////////////////////////////
    //
    const onReview = ()=>{
        const spreaderSelectTabElm = document.getElementById('spreadSelectTab');
        spreaderSelectTabElm.style.display='none';
        const spreaderReviewTabElm = document.getElementById('spreadReviewTab');
        spreaderReviewTabElm.style.display='block';


        const reviewSelections = document.getElementById(spreadingDlgRt+"SpreaderSelectionReview");

        //Clear any old selections
        if (0!==reviewSelections.options.length) {
            for (let i = reviewSelections.options.length-1; i >= 0; i--) {
                reviewSelections.remove(i);
            }
        }

        //Add new options
        const selectedOptions = document.getElementById(`${spreadingDlgRt}SpreaderSelection`).options
        for (const anOpt of selectedOptions) {
            if (anOpt.selected) {
                const newOpt = document.createElement("option");
                newOpt.text = anOpt.text;
                newOpt.value = anOpt.value;
                reviewSelections.add(newOpt);
            }
        }
        if (0===reviewSelections.options.length) { return; }

        document.getElementById("spreadersSaveBtn").classList.remove("make-disabled");
    };
    
    /////////////////////////////////////
    //
    const spreaderCandidates = [];
    for (const userInfo of frConfig.users({doFilterOutAdmins: true})) {
        spreaderCandidates.push(
            <option key={userInfo[0]} value={userInfo[0]} onMouseDown={handleSelection}>
                {userInfo[1]}
            </option>);
    }

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
                                <div className="col-sm" id="spreadSelectTab">
                                    <label htmlFor={spreadingDlgRt+"SpreaderSelection"}>
                                        Select Spreaders
                                    </label>
                                    <select className="form-select" id={spreadingDlgRt+"SpreaderSelection"}
                                            multiple size="10" aria-label="Select Spreaders">
                                        {spreaderCandidates}
                                    </select>
                                </div>
                                <div className="col-sm" id="spreadReviewTab" style={{display: 'none'}}>
                                    <label htmlFor={spreadingDlgRt+"SpreaderSelectionReview"}>
                                        Review Spreaders
                                    </label>
                                    <select className="form-select" id={spreadingDlgRt+"SpreaderSelectionReview"}
                                            multiple size="10" disabled aria-label="Review Spreaders">
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div className="modal-footer">
                        <button type="button float-start" className="btn btn-secondary"
                                data-bs-dismiss="modal">Cancel</button>
                        <div className="btn-group" role="group" aria-label="Spreader Selection Group">
                            <input type="radio" className="btn-check" name="btnradio" id="spreadersSelectBtn"
                                   autoComplete="off" defaultChecked onClick={onSelect}/>
                            <label className="btn btn-outline-primary" htmlFor="spreadersSelectBtn" >Select</label>

                            <input type="radio" className="btn-check" name="btnradio" id="spreadersReviewBtn"
                                   autoComplete="off" onClick={onReview}/>
                            <label className="btn btn-outline-primary" htmlFor="spreadersReviewBtn" >Review</label>
                        </div>
                        <button type="button" className="btn btn-primary make-disabled" id="spreadersSaveBtn">
                            <span className="spinner-border spinner-border-sm me-1" role="status"
                                  aria-hidden="true" id="spreadingSubmitBtnSpinny" style={{display: "none"}} />
                            Submit
                        </button>
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
                    <li className="list-group-item me-3">
                        <label className="text-muted pe-2">Report View:</label>
                        <div className="d-inline" id="reportViewLabel">Default</div>
                    </li>
                    <li className="list-group-item" id="orderOwnerLabel">
                        <label className="text-muted pe-2">Showing Orders for:</label>
                        <div className="d-inline" id="reportViewOrderOwner">{fullName}</div>
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

            if (lastAuthenticatedUser !== auth.getCurrentUserId()) {
                // We had a user login change
                reportViews.resetToDefault();
                lastAuthenticatedUser = auth.getCurrentUserId();
            }

            const frConfig = getFundraiserConfig();

            console.log("loaded FrConfig");

            console.log("Loading Gen Card Body");
            setCardBody(genCardBody(frConfig));
            setDeleteDlg(genDeleteDlg());
            setSpreadDlg(genSpreadingDlg(frConfig));
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
            <div className="modal fade" id="confirmDlg"
                 tabIndex="-1" role="dialog" aria-labelledby="deleteOrderDlgTitle" aria-hidden="true">
                <div className="modal-dialog modal-dialog-centered" role="document">
                    <div className="modal-content">
                        <div className="modal-header">
                            <h5 className="modal-title" id="deleteOrderDlgLongTitle">
                                Confirmation Requested
                            </h5>
                            <button type="button" className="close" data-bs-dismiss="modal" aria-label="Close">
                                <span aria-hidden="true">&times;</span>
                            </button>
                        </div>
                        <div className="modal-body">
                            <small id="confirmDeleteOrderHelp" className="form-text text-muted">
                                Do you wish to continue?
                            </small>

                        </div>
                        <div className="modal-footer">
                            <button type="button" className="btn btn-secondary"
                                    data-bs-dismiss="modal">Cancel</button>
                            <div className="modal-footer">
                                <button type="button" className="btn btn-primary" id="confirmDlgBtn">
                                    <span className="spinner-border spinner-border-sm me-1" role="status"
                                          aria-hidden="true" id="confirmDlgBtnSpinny"
                                          style={{display: "none"}} />
                                    Save
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
