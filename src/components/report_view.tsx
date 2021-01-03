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
    private currentDataset_: Array<any> = undefined;


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
                console.log(`Report contains ${this.currentDataset_.length} rows`);
            })
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

                    const selectedSpreaders = document.getElementById(spreadingDlgRt+"SpreaderSelection");

                    const spreaders=[];
                    for (const anOpt of selectedSpreaders.options) {
                        if (anOpt.selected) {
                            spreaders.push(anOpt.value);
                        }
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
                    (order.isVerified?"True":"False"),
                    orderOwner,
                    this.getActionButtons(order, frConfig)
                ]);
                this.currentDataset_.push(orderDataItem);
            }
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

        this.currentDataTable_ = this.genDataTable(this.currentDataset_, tableColumns);

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

const reportViews: ReportViews = new ReportViews();

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
        jQuery(reviewSelections).empty();

        const genLi = (label)=>{
            const li = document.createElement("li");
            li.appendChild(document.createTextNode(label));
            li.classList.add('list-group-item');
            return li;
        };

        //Add new options
        const selectedOptions = document.getElementById(`${spreadingDlgRt}SpreaderSelection`).options
        for (const anOpt of selectedOptions) {
            if (anOpt.selected) {
                const newOpt = genLi(anOpt.text);
                reviewSelections.appendChild(newOpt);
            }
        }
        if (0===reviewSelections.querySelectorAll("li").length) {
            reviewSelections.style.display = 'none';
            document.getElementById(spreadingDlgRt+"SpreaderNoSelectionReview").style.display = 'block';
        } else {
            reviewSelections.style.display = 'block';
            document.getElementById(spreadingDlgRt+"SpreaderNoSelectionReview").style.display = 'none';
        }

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
