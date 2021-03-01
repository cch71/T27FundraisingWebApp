import React from "react";
import {orderDb, OrderListItem} from "../js/ordersdb";
import currency from "currency.js";
import auth from "../js/auth";
import { navigate } from "gatsby";
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config";
import bootstrapIconSprite from "bootstrap-icons/bootstrap-icons.svg";

const trashImg = bootstrapIconSprite + "#trash";
const pencilImg = bootstrapIconSprite + "#pencil";
const eyeImg = bootstrapIconSprite + "#eye";
const spreadImg = bootstrapIconSprite + "#layout-wtf";

const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });
const spreadingDlgRt = 'spreadingDlg';

////////////////////////////////////////////////////////////////////
//
const resetSpreaderDlg = ()=>{
    // reset dlg to default state.  I.E. have selection btn be active view
    // and reset hidden currentOrderId, currentOrderOwner
    const reviewSelections = document.getElementById(spreadingDlgRt+"SpreaderSelectionReview");
    reviewSelections.style.display='none';
    document.getElementById(spreadingDlgRt+"SpreaderNoSelectionReview").style.display='none';
    document.getElementById('spreadingSubmitBtnSpinny').style.display = 'none';
    jQuery(reviewSelections).empty();

    for (const cb of jQuery(`#${spreadingDlgRt}SpreaderSelection input:checkbox:checked`)) {
        //console.log(`Removing Checked: ${cb.id}: ${cb.parentNode.innerText}`);
        cb.parentNode.classList.remove("active");
        cb.checked = false;
    }

    jQuery('#spreadSelectTab').show();
    jQuery('#spreadReviewTab').hide();

    document.getElementById("spreadersSelectBtn").checked = true;
    document.getElementById("spreadersSaveBtn").classList.add("make-disabled");
};


////////////////////////////////////////////////////////////////////
//
class ReportViews {
    private currentView_: string;
    private currentUserId_: string;
    private currentDataTable_: any = undefined;
    private currentDataset_: Array<any> = undefined;
    private reportHeaders_: Array<string> = undefined;



    constructor() {
        /* console.log("Constructing...");
         * window.addEventListener('resize', ()=>{
         *     console.log(`Evt Scadreen Height ${screen.height} window innerHeight: ${window.innerHeight}`);
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
            views.push(['Full', undefined]);
            if ('mulch' === frConfig.kind()) {
                views.push(['Spreading Jobs', 'SpreadingJobs']);
            }
            views.push(['Verify Orders', 'VerifyOrders']);
            if ('mulch' === frConfig.kind()) {
                views.push(['Distribution Points', 'DistributionPoints']);
            }
            
            for (const userInfo of frConfig.users()) {
                users.push(userInfo);
            }
            users.push(['any', 'All']);
        } else {
            views.push(['Default', undefined]);
            views.push(['Full', undefined]);
            if ('mulch' === frConfig.kind()) {
                views.push(['Spreading Jobs', 'SpreadingJobs']);
            }

            const fullName = frConfig.getUserNameFromId(auth.getCurrentUserId())
            const uid = auth.getCurrentUserId();
            users.push([uid, fullName]);
        }
        return [views, users];
    }


    ////////////////////////////////////////////////////////////////////
    //
    dataSetChanged() {
        console.log("Resetting Dataset");
        delete this.currentDataset_;
    }

    ////////////////////////////////////////////////////////////////////
    //
    resetToDefault() {
        console.log("Resetting Report Data To Defaults");
        this.currentUserId_ = auth.getCurrentUserId();
        this.currentView_ = 'Default';
        delete this.currentDataTable_;
        delete this.currentDataset_;
    }

    ////////////////////////////////////////////////////////////////////
    //
    getCurrentUserId() { return this.currentUserId_; }

    ////////////////////////////////////////////////////////////////////
    //
    getCurrentView() { return this.currentView_; }

    ////////////////////////////////////////////////////////////////////
    //
    // interface ReportsShowViewParams {
    //    userId?: string;
    //}
    showView(view: string, frConfig: FundraiserConfig, params?: any /* ReportsShowViewParams*/) {
        const asyncShow = async () => {
            const userId = params?.userId;
            const spinnerElm = document.getElementById('orderLoadingSpinner');
            if (spinnerElm) {
                spinnerElm.classList.remove('visually-hidden');
            }

            if (jQuery.fn.dataTable.isDataTable('#reportsTable table')) {
                if (view === this.currentView_ && userId === this.currentUserId_) { return; }

                console.log("Resetting Report Data Table");
                jQuery('#reportsTable table').DataTable().clear();
                jQuery('#reportsTable table').DataTable().destroy();
                jQuery('#reportsTable table').empty();
                this.resetToDefault();
            }

            console.log(`Current View: ${this.currentView_} New View: ${view}`);
            console.log(`Current User: ${this.currentUserId_} New User: ${userId}`);
            this.currentView_ = view;
            this.currentUserId_ = userId;

            if(typeof this[`show${view}`] === 'function') {
                await this[`show${view}`](frConfig, userId);
            } else {
                throw new Error(`Report View Type: ${view} not found`);
            }

            if (spinnerElm) {
                spinnerElm.classList.add('visually-hidden');
            }
        }

        asyncShow()
            .then(()=>{
                console.log(`Report contains ${this.currentDataset_?.length} rows`);
            })
            .catch((err: any)=>{
                if ('Invalid Session'===err.message) {
                    navigate('/')
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
                `<button type="button" class="btn btn-outline-info me-1 order-spread-btn"`+
                ` data-bs-toggle="tooltip" title="Select Spreaders" data-bs-placement="left">` +
                `<svg class="bi" fill="currentColor"><use xlink:href="${spreadImg}" /></svg></button>`;
        }

        if (frConfig.isEditableDeliveryDate(order.deliveryId)) {
            htmlStr +=
                `<button type="button" class="btn btn-outline-info me-1 order-edt-btn"` +
                ` data-bs-toggle="tooltip" title="Edit Order" data-bs-placement="left">` +
                `<svg class="bi" fill="currentColor"><use xlink:href="${pencilImg}" /></svg></button>` +
                `<button type="button" class="btn btn-outline-danger order-del-btn"` +
                ` data-bs-toggle="tooltip" title="Delete Order" data-bs-placement="left">` +
                `<svg class="bi" fill="currentColor"><use xlink:href="${trashImg}" /></svg></button>`;
        } else {
            htmlStr +=
                `<button type="button" class="btn btn-outline-info me-1 order-view-btn"` +
                ` data-bs-toggle="tooltip" title="View Order" data-bs-placement="left">` +
                `<svg class="bi" fill="currentColor"><use xlink:href="${eyeImg}" /></svg></button>`;
        }

        htmlStr += `</div>`;
        return htmlStr;
    }

    ////////////////////////////////////////////////////////////////////
    //
    private genDataTable(orderDataSet: any, tableColumns: any) {
        //console.log(`Screen Height ${screen.height} window innerHeight: ${window.innerHeight}`);
        return jQuery('#reportsTable table').DataTable({
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
        jQuery('#reportsTable').find('.order-edt-btn').off('click');
        jQuery('#reportsTable').find('.order-view-btn').off('click');
        jQuery('#reportsTable').find('.order-spread-btn').off('click');
        jQuery('#reportsTable').find('.order-del-btn').off('click');
        jQuery('#reportsTable').find('.order-vrfy-switch').off('change');

        // Handle on Edit Scenario
        jQuery('#reportsTable').find('.order-edt-btn').on('click', (event: any)=>{
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
        jQuery('#reportsTable').find('.order-view-btn').on('click', (event: any)=>{
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
        jQuery('#reportsTable').find('.order-spread-btn').on('click', (event: any)=>{
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

            for(const spreader of rowData[spreadersIdx]) {
                const cb = jQuery(`#spreadSelUsr${spreader}`)[0];
                cb.parentNode.classList.add("active");
                cb.checked = true;
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

                    const spreaders=[];
                    for (const cb of jQuery(`#${spreadingDlgRt}SpreaderSelection input:checkbox:checked`)) {
                        //console.log(`Found CB: ${cb.dataset.uid}: ${cb.parentNode.innerText}`);
                        spreaders.push(cb.dataset.uid);
                    }

                    // submit order update
                    console.log(`Submitting spreading job for ` +
                                `${orderId}:${orderOwner}: ${JSON.stringify(spreaders)}`);

                    // Start submit spnner
                    const submitSpinner = document.getElementById('spreadingSubmitBtnSpinny');
                    submitSpinner.style.display = "inline-block";
                    document.getElementById('spreadersSaveBtn').disabled = true;
                    orderDb.submitSpreadingComplete({
                        orderOwner: orderOwner,
                        orderId: orderId,
                        spreaders: spreaders
                    }).then(()=>{
                        document.getElementById('spreadersSaveBtn').disabled = false;
                        resetSpreaderDlg();
                        spreadOrderDlg.hide();
                        rowData[spreadersIdx] = spreaders;
                        row.data(rowData).draw();
                    }).catch((err:any)=>{
                        document.getElementById('spreadersSaveBtn').disabled = false;
                        if ('Invalid Session'===err.message) {
                            navigate('/')
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
        jQuery('#reportsTable').find('.order-del-btn').on('click', (event: any)=>{
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

        jQuery('#reportsTable').find('.order-vrfy-switch').on('change', (event: any)=>{
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
                        if ('Invalid Session'===err.message) {
                            navigate('/')
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

        //$('[data-bs-toggle="tooltip"]').tooltip();


    }

    ////////////////////////////////////////////////////////////////////
    //
    private async showDefault(frConfig: FundraiserConfig, userId?: string) {

        const currentUserId =  auth.getCurrentUserId();
        if (!userId) { userId = currentUserId; }

        if (!this.currentDataset_) {
            // Build query fields
            const fieldNames = ["orderId", "firstName", "lastName", "deliveryId"];
            if ('mulch' === frConfig.kind()) {
                fieldNames.push("spreaders");
                fieldNames.push("products.spreading");
            }

            if ('any'===userId) {
                fieldNames.push("orderOwner");
            }


            const orders = await orderDb.query({fields: fieldNames, orderOwner: userId});
            //console.log(`Default Orders Page: ${JSON.stringify(orders)}`);

            // Fill out rows of data
            this.currentDataset_ = [];
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
                this.currentDataset_.push(orderDataItem);
            }

            this.reportHeaders_ = ["OrderId", "CustomerName", "DeliveryDate"];
            if ('mulch' === frConfig.kind()) {
                this.reportHeaders_.push("Spreaders");
                this.reportHeaders_.push("BagsToSpread");
            }
            this.reportHeaders_.push("orderOwner");
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

        this.currentDataTable_ = this.genDataTable(this.currentDataset_, tableColumns);

        //this.registerActionButtonHandlers();
    }

    ////////////////////////////////////////////////////////////////////
    //
    private async showSpreadingJobs(frConfig: FundraiserConfig, userId?: string) {

        const currentUserId =  auth.getCurrentUserId();
        if (!userId) { userId = currentUserId; }
        if (!this.currentDataset_) {
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


            const orders = await orderDb.query({fields: fieldNames, orderOwner: userId});
            //console.log(`Default Orders Page: ${JSON.stringify(orders)}`);

            // Fill out rows of data
            this.currentDataset_ = [];
            const nowDate = Date.now();
            for (const order of orders) {
                const deliveryDate = order.deliveryId?frConfig.deliveryDateFromId(order.deliveryId):'donation';

                if (!(order.products?.spreading &&
                      deliveryDate!=='donation' &&
                      nowDate > Date.parse(deliveryDate)))
                {
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
                this.currentDataset_.push(orderDataItem);
            }

            this.reportHeaders_ = ["OrderId", "CustomerName", "DeliveryDate", "SpecialInstructions", "Address"];
            if ('mulch' === frConfig.kind()) {
                this.reportHeaders_.push("Neighborhood");
                this.reportHeaders_.push("Spreaders");
                this.reportHeaders_.push("BagsToSpread");
            }
            this.reportHeaders_.push("orderOwner");
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

        this.currentDataTable_ = this.genDataTable(this.currentDataset_, tableColumns);

        //this.registerActionButtonHandlers();
    }

    ////////////////////////////////////////////////////////////////////
    //
    private async showVerifyOrders(frConfig: FundraiserConfig, userId?: string) {

        const currentUserId =  auth.getCurrentUserId();
        if (!userId) { userId = currentUserId; }
        if (!this.currentDataset_) {
            // Build query fields
            const fieldNames = ["orderId", "firstName", "lastName", "deliveryId",
                                "totalAmt", "cashPaid", "checkPaid", "checkNums", "isVerified"];


            if ('any'===userId) {
                fieldNames.push("orderOwner");
            }


            const orders = await orderDb.query({fields: fieldNames, orderOwner: userId});
            //console.log(`Verify Orders Page: ${JSON.stringify(orders)}`);

            /* const htmlValidateSwitch =
             *     `<div class="form-check form-switch">` +
             *     `<input class="form-check-input order-vrfy-switch" type="checkbox" />` +
             *     `</div>`;
             */
            // Fill out rows of data
            this.currentDataset_ = [];
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
                this.currentDataset_.push(orderDataItem);
            }

            this.reportHeaders_ = ["OrderId", "CustomerName", "DeliveryDate", "TotalAmount", "CashCollected",
                                   "ChecksCollected", "CheckNumbers", "OrderOwner", "IsVerified"];
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

        this.currentDataTable_ = this.genDataTable(this.currentDataset_, tableColumns);

        //this.registerActionButtonHandlers();
    }

    ////////////////////////////////////////////////////////////////////
    //
    private async showFull(frConfig: FundraiserConfig, userId?: string) {

        const currentUserId =  auth.getCurrentUserId();
        if (!userId) { userId = currentUserId; }

        if (!this.currentDataset_) {
            const orders = await orderDb.query({orderOwner: userId});

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
            this.currentDataset_ = [];
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
                    orderOwner,
                    this.getActionButtons(order, frConfig)
                ]);
                this.currentDataset_.push(orderDataItem);
            }

            this.reportHeaders_ = ["OrderId", "CustomerName", "Phone", "Email", "Address1", "Address2",
                                   "DeliveryDate"];
            if ('mulch' === frConfig.kind()) {
                this.reportHeaders_ = this.reportHeaders_.concat([
                    "Neighborhood", "Spreaders", "BagsToSpread", "BagsPurchased"]);
            }

            this.reportHeaders_ = this.reportHeaders_.concat([
                "SpecialInstructions", "IsVerified", "IsMoneyCollected", "Donations", "CashCollected",
                "ChecksCollected", "CheckNumbers", "TotalAmount", "OrderOwner"]);

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

        this.currentDataTable_ = this.genDataTable(this.currentDataset_, tableColumns);

        //this.registerActionButtonHandlers();
    }

    ////////////////////////////////////////////////////////////////////
    //
    private async showDistributionPoints(frConfig: FundraiserConfig, userId?: string) {

        if (!this.currentDataset_) {
            const orders = await orderDb.query({orderOwner: "any"});

            //console.log(`Full Orders Page: ${JSON.stringify(orders)}`);

            const getVal = (fld?: any)=>{
                if (undefined===fld) {
                    return 0;
                }
                return parseInt(fld);
            };

            // Fill out rows of data
            this.currentDataset_ = [];
            const orderByDate = {};
            const distPtsSet = new Set();

            for (const order of orders) {
                if (!order.deliveryId) { continue;}
                const deliveryDate = frConfig.deliveryDateFromId(order.deliveryId);
                if (!orderByDate[deliveryDate]) {
                    orderByDate[deliveryDate] = { "totalBags": 0};
                }
                const bags = getVal(order.products?.bags);
                if (0===bags) { continue; }
                const distPt = frConfig.getDistributionPoint(order.neighborhood);
                distPtsSet.add(distPt);
                if (!orderByDate[deliveryDate][distPt]) {
                    orderByDate[deliveryDate][distPt] = {"bags": 0};
                }
                orderByDate[deliveryDate][distPt].bags += bags;
                orderByDate[deliveryDate].totalBags += bags;
            }

            const dPointsArray = Array.from(distPtsSet);
            this.reportHeaders_ = ["DeliveryDate", "TotalBags"];
            this.reportHeaders_ = this.reportHeaders_.concat(dPointsArray);

            //console.log(`Calc Report Val:\n${JSON.stringify(orderByDate, null, '\t')}`);
            // Now normalize to table
            for (const dDate of Object.keys(orderByDate)){
                const dataItem=[dDate, orderByDate[dDate].totalBags];
                for (const dPoint of dPointsArray) {
                    if (orderByDate[dDate].hasOwnProperty(dPoint)) {
                        dataItem.push(orderByDate[dDate][dPoint].bags);
                    } else {
                        dataItem.push(0);
                    }
                }
                this.currentDataset_.push(dataItem);
            }
        }

        const tableColumns =[];
        for (const header of this.reportHeaders_) {
            tableColumns.push({title: header, className: 'all'});
        }
        //tableColumns[0]['className'] = "all";

        this.currentDataTable_ = this.genDataTable(this.currentDataset_, tableColumns);
    }

    
    ////////////////////////////////////////////////////////////////////
    //
    genCsvFromCurrent(): string {
        if (!this.currentDataTable_) { throw new Error("Table isn't found"); }
        let csvFileData = [];

        //Get Data
        const data = this.currentDataTable_.data().toArray();
        let csvData = [];
        data.forEach((row, _)=>{
            let csvRow = [];
            row.forEach((column, _)=>{
                csvRow.push(column);
            });
            csvRow.splice(-1,1);
            csvData.push(csvRow);
        });

        return Papa.unparse({
	          "fields": this.reportHeaders_,
	          "data": csvData,
        });

        //console.log(`${JSON.stringify(csvFileData, null, '\t')}`);
    }

}

const reportViews: ReportViews = new ReportViews();

////////////////////////////////////////////////////////////////////
//
const genSpreadingDlg = (frConfig: FundraiserConfig) => {

    /////////////////////////////////////
    //
    const handleSelection = (evt)=>{
        if (evt.currentTarget.checked) {
            //console.log(`Adding Checked ${evt.currentTarget.checked}`);
            evt.currentTarget.parentNode.classList.add("active");
            //evt.currentTarget.parentNode.style.background = "#0d6efd";
            //evt.currentTarget.parentNode.style.color = "white";
        } else {
            //console.log(`Remving Checked ${evt.currentTarget.checked}`);
            evt.currentTarget.parentNode.classList.remove("active");
            //evt.currentTarget.parentNode.style.background = "white";
            //evt.currentTarget.parentNode.style.color = "#0d6efd";
        }
    };

    /////////////////////////////////////
    //
    const onSelect = ()=>{
        jQuery('#spreadSelectTab').show();
        jQuery('#spreadReviewTab').hide();

        document.getElementById("spreadersSaveBtn").classList.add("make-disabled");
    };

    /////////////////////////////////////
    //
    const onReview = ()=>{
        jQuery('#spreadSelectTab').hide();
        jQuery('#spreadReviewTab').show();

        const reviewSelections = document.getElementById(spreadingDlgRt+"SpreaderSelectionReview");

        //Clear any old selections
        jQuery(reviewSelections).empty();

        const genLi = (label)=>{
            const li = document.createElement("li");
            li.appendChild(document.createTextNode(label));
            li.classList.add('list-group-item');
            return li;
        };

        //Add new options
        for (const cb of jQuery(`#${spreadingDlgRt}SpreaderSelection input:checkbox:checked`)) {
            //console.log(`Found CB: ${cb.id}: ${cb.parentNode.innerText}`);
            const newOpt = genLi(cb.parentNode.innerText);
            reviewSelections.appendChild(newOpt);
        }

        if (0===reviewSelections.querySelectorAll("li").length) {
            jQuery(reviewSelections).hide();
            jQuery(`#${spreadingDlgRt}SpreaderNoSelectionReview`).show();
        } else {
            jQuery(reviewSelections).show();
            jQuery(`#${spreadingDlgRt}SpreaderNoSelectionReview`).hide();
        }

        document.getElementById("spreadersSaveBtn").classList.remove("make-disabled");
    };

    /////////////////////////////////////
    //
    //
    const spreaderCandidates = [];
    for (const userInfo of frConfig.users({doFilterOutAdmins: true})) {
        spreaderCandidates.push(
            <label key={userInfo[0]} className="btn btn-outline-primary" >
                {userInfo[1]}
                <input type="checkbox" className="btn-check" data-uid={userInfo[0]}
                       id={`spreadSelUsr${userInfo[0]}`} onChange={handleSelection} autoComplete="off" />
            </label>
        );
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
                                    <div className="btn-group-vertical overflow-auto" role="group"
                                         id={spreadingDlgRt+"SpreaderSelection"} aria-label="Select Spreaders">
                                        {spreaderCandidates}
                                    </div>
                                </div>
                                <div className="col-sm" id="spreadReviewTab" style={{display: 'none'}}>
                                    <label htmlFor={spreadingDlgRt+"SpreaderSelectionReview"}>
                                        Review Spreaders
                                    </label>
                                    <ul className="list-group" id={spreadingDlgRt+"SpreaderSelectionReview"}>
                                    </ul>
                                    <div className="alert alert-danger"
                                         id={spreadingDlgRt+"SpreaderNoSelectionReview"}>
                                        <h6>No spreaders were selected.
                                            Submitting this will mark this order as not spread yet.</h6>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div className="modal-footer">
                        <button type="button" className="btn btn-secondary"
                                data-bs-dismiss="modal">Cancel</button>
                        <div className="btn-group" role="group" aria-label="Spreader Selection Group">
                            <input type="radio" className="btn-check" name="btnradio" id="spreadersSelectBtn"
                                   autoComplete="off" defaultChecked onClick={onSelect}/>
                            <label className="btn btn-outline-primary" htmlFor="spreadersSelectBtn" >1. Select</label>

                            <input type="radio" className="btn-check" name="btnradio" id="spreadersReviewBtn"
                                   autoComplete="off" onClick={onReview}/>
                            <label className="btn btn-outline-primary" htmlFor="spreadersReviewBtn" >2. Review</label>
                            <input type="radio" className="btn-check make-disabled" name="btnradio" id="spreadersSaveBtn"
                                   autoComplete="off"/>
                            <label className="btn btn-outline-primary" htmlFor="spreadersSaveBtn" >
                                3. Submit
                                <span className="spinner-border spinner-border-sm me-1" role="status"
                                      aria-hidden="true" id="spreadingSubmitBtnSpinny" style={{display: "none"}} />
                            </label>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

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
                        <button type="button" className="btn btn-secondary" data-bs-dismiss="modal">Cancel</button>
                        <button type="button" disabled className="btn btn-primary" id="deleteDlgBtn">
                            Delete Order
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
};

export {reportViews, genDeleteDlg, genSpreadingDlg};
