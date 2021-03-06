import React, { useState, useEffect } from "react"
import { Router } from '@reach/router';
import {orderDb, Order} from "../js/ordersdb"
import auth from "../js/auth"
import {reportViews} from "../components/report_view"
import { navigate } from "gatsby"
import currency from "currency.js"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"
import {onCurrencyFieldKeyPress, onCheckNumsKeyPress, moneyFloor, saveCurrentOrder} from "../js/utils"
import bootstrapIconSprite from "bootstrap-icons/bootstrap-icons.svg";
const trashImg = bootstrapIconSprite + "#trash";
const pencilImg = bootstrapIconSprite + "#pencil";
const eyeImg = bootstrapIconSprite + "#eye";
const alertImg = bootstrapIconSprite + "#exclamation-triangle-fill";

const USD = (value) => currency(value, { symbol: "$", precision: 2 });

////////////////////////////////////////////////////////
//
const validateOrderForm = (currentOrder: Order) => {
    //  Goes through required fields and verifies that they are valid
    const validatePayment = async ()=>{
        const amountDue = currency(document.getElementById('orderAmountDue').innerText);
        const amountPaid = currency(document.getElementById('orderAmountPaid').innerText);
        const checksPaid = currency(document.getElementById('formCheckPaid').value);
        const checkNumField = document.getElementById('formCheckNumbers');
        const checkNumFieldVal = checkNumField.value;
        const isCheckNumGood = (0.0<checksPaid.value)?(0<checkNumFieldVal.length):true;
        const isCollectLaterChecked = document.getElementById('formCollectLater').checked;
        const isPaidChecked = amountDue.value===amountPaid.value;

        //console.log(`AD: ${amountDue.value} AP: ${amountPaid.value}`);

        if ((isPaidChecked || isCollectLaterChecked) && isCheckNumGood) {
            document.getElementById('totalsFormRow').classList.remove('is-invalid');
            checkNumField.classList.remove('is-invalid');
            return true;
        }

        document.getElementById('totalsFormRow').classList.add('is-invalid');
        if (!isCheckNumGood) { checkNumField.classList.add('is-invalid'); }
        return false;
    };

    const validateRequiredFormFields = async ()=>{
        let isValid = true;
        const formElms = document.querySelector("#newOrEditOrderForm")
                                 .querySelectorAll('[required]');
        Array.prototype.slice.call(formElms).forEach((aform) => {
            if (!aform.checkValidity()) {
                aform.classList.add('is-invalid');
                isValid = false;
            } else {
                aform.classList.remove('is-invalid');
            }
        });

        return isValid;
    };

    const validateProducts = async ()=>{
        if (currentOrder.products || currentOrder.donation) {
            document.getElementById('productList').classList.remove('is-invalid');
            return true;
        }
        document.getElementById('productList').classList.add('is-invalid');
        return false;
    };

    return [validatePayment(), validateRequiredFormFields(), validateProducts()];
}

////////////////////////////////////////////////////////
//
const populateForm = async (currentOrder: Order, setFormFields: any, isAdmin: boolean, isOrderReadOnly: boolean): any =>{

    const frConfig: FundraiserConfig = getFundraiserConfig();
    if (!frConfig) {
        alert("Failed to load fundraiser config");
        return(<div/>);
    }

    ////////////////////////////////////////////////////////
    // Handle Form Submission
    const onFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();

        (document.getElementById('formOrderSubmit') as HTMLButtonElement).disabled = true;
        (document.getElementById('formOrderSubmitSpinner') as HTMLButtonElement).style.display = "inline-block";
        (document.getElementById('formOrderCancel') as HTMLButtonElement).disabled = true;


        console.log(`Submitting Active Order`);
        saveCurrentOrder();

        // Validate Form
        const reenableSubmitButton = () => {
            (document.getElementById('formOrderSubmit') as HTMLButtonElement).disabled = false;
            (document.getElementById('formOrderSubmitSpinner') as HTMLButtonElement).style.display = "none";
            (document.getElementById('formOrderCancel') as HTMLButtonElement).disabled = false;
        };

        // If everything vlidates then submit
        Promise.all(validateOrderForm(currentOrder))
               .then((results)=>{
                   console.log(`Results: ${JSON.stringify(results)}`);
                   if (results[0] && results[1] && results[2]) {
                       // If we got here they we are good to submit form
                       currentOrder.isVerified = false;
                       const isLoadedFromDb = currentOrder.meta?.isLoadedFromDb;
                       orderDb.submitActiveOrder().then(()=>{
                           reportViews.dataSetChanged();
                           if (isLoadedFromDb) {
                               navigate('/reports/');
                           } else {
                               navigate('/');
                           }
                       }).catch((err:any)=>{
                           if ('Invalid Session'===err.message) {
                               navigate('/')
                           } else {
                               reenableSubmitButton();
                               const errStr = `Failed submitting order: ${JSON.stringify(err)}`;
                               console.log(errStr);
                               alert(errStr);
                               throw err;
                           }
                       });
                   } else {
                       console.log("Not all fields are validated");
                       reenableSubmitButton();
                   }
               });
    }

    // Add New Product Order
    const onAddOrder = async (event: any)=>{
        event.preventDefault();
        event.stopPropagation();

        saveCurrentOrder();

        const btn = event.currentTarget;

        let pageState = {state: {}};
        if (btn.dataset && btn.dataset.deliveryid) {
            const deliveryId = btn.dataset.deliveryid;
            console.log(`Add New Fundraising Order for ${deliveryId}`);
            pageState.state['deliveryId'] = deliveryId;
            pageState.state['isOrderReadOnly'] = isOrderReadOnly;
        }
        pageState.state['isAdmin'] = await auth.isCurrentUserAdmin();

        navigate('/add_products_order/', pageState);
    };

    ////////////////////////////////////////////////////////
    // Add Donation to order
    const onAddDonation = (event: any)=>{
        event.preventDefault();
        event.stopPropagation();
        console.log(`Adding New Donation`);

        saveCurrentOrder()
        navigate('/add_donations/');
    };

    ////////////////////////////////////////////////////////
    // Discard the order and go back to home
    const onDiscardOrder = ()=>{
        const isLoadedFromDb = currentOrder.meta?.isLoadedFromDb;
        orderDb.setActiveOrder();
        if (isLoadedFromDb) {
            navigate('/reports/');
        } else {
            navigate('/');
        }
    };

    ////////////////////////////////////////////////////////
    //
    const calcCurrentOrderCost=()=>{
        let totalCost = currency(0.0);
        if (currentOrder.donation) { totalCost = totalCost.add(currentOrder.donation); }
        if (currentOrder.productsCost) { totalCost = totalCost.add(currentOrder.productsCost); }
        return totalCost;
    };

    ////////////////////////////////////////////////////////
    // Recalculate Total due dynamically based on changes to order status
    const recalculateTotalDue = ()=> {
        const totalDue = calcCurrentOrderCost();
        const totElm = document.getElementById('orderAmountDue');
        if (totElm) {
            totElm.innerText = `${USD(totalDue).format()}`;
        }
    }

    ////////////////////////////////////////////////////////
    //
    const recalculateTotalPaid = ()=> {
        const [cash, isCashChanged] =
            moneyFloor((document.getElementById('formCashPaid') as HTMLInputElement).value);
        const [checks, isChecksChanged] =
            moneyFloor((document.getElementById('formCheckPaid') as HTMLInputElement).value);

        if (isCashChanged) {
            (document.getElementById('formCashPaid') as HTMLInputElement).value = cash.toString();
        }
        if (isChecksChanged) {
            (document.getElementById('formCheckPaid') as HTMLInputElement).value = checks.toString();
        }

        const totPaid = cash.add(checks);
        const totElm = document.getElementById('orderAmountPaid');
        if (null!==totElm) {
            totElm.innerText = `${USD(totPaid).format()}`;
        }

        const collectLaterElm = (document.getElementById('formCollectLater') as HTMLInputElement);
        if (0 < totPaid.value) {
            if (collectLaterElm.checked) {
                collectLaterElm.checked = false;
            }
            collectLaterElm.disabled = true;
        } else {
            collectLaterElm.disabled = false;
        }



    }

    ////////////////////////////////////////////////////////
    // Create delivery status buttons
    const populateOrdersList = (): Array<any> => {

        const ordersByDeliveryBtns = []

        //  Create the ADD buttons and based on existing order deside visibility
        // Figure out visibility
        const addExistingOrderButton = (deliveryId: string, deliveryLabel: string, productsCost: currency)=>{
            //console.log(`Adding Order Type for DDay: ${deliveryId}`);
            const foundTag = `found-${deliveryId}`
            const orderTotalStr = `${deliveryLabel} Amount: ${USD(productsCost).format()} `;

            const onClickHandler = ("donation" === deliveryId)? onAddDonation : onAddOrder;
            ////////////////////////////////////////////////////////
            // On Delete Order Re-enable button
            const onDeleteOrder = (event)=>{
                event.preventDefault();
                event.stopPropagation();

                const deliveryIdToDel = event.currentTarget.dataset.deliveryid;

                console.log(`Deleting Order for ${deliveryIdToDel}`);
                if ('donation' === deliveryIdToDel) {
                    delete currentOrder.donation;
                } else {
                    delete currentOrder.products;
                    delete currentOrder.productsCost;
                    delete currentOrder.deliveryId;
                }

                document.getElementById(foundTag).style.display = "none";
                if ('donation' === deliveryIdToDel) {
                    document.getElementById('addDonationBtnLi').style.display = "block";
                } else {
                    document.getElementById('addProductBtnLi').style.display = "block";
                }

                recalculateTotalDue();
            }

            if (currentOrder.meta?.isReadOnly) {
                ordersByDeliveryBtns.push(
                    <li className="list-group-item" id={foundTag} key={foundTag}>
                        {orderTotalStr}
                        <button className="btn btn-outline-info float-end order-edt-btn"
                                data-deliveryid={deliveryId} onClick={onClickHandler}>
                            <svg className="bi" fill="currentColor"><use xlinkHref={eyeImg}/></svg>
                        </button>
                    </li>
                );
            } else {
                ordersByDeliveryBtns.push(
                    <li className="list-group-item" id={foundTag} key={foundTag}>
                        {orderTotalStr}
                        <button className="btn btn-outline-danger mx-1 float-end order-del-btn"
                                data-deliveryid={deliveryId} onClick={onDeleteOrder}>
                            <svg className="bi" fill="currentColor"><use xlinkHref={trashImg}/></svg>
                        </button>
                        <button className="btn btn-outline-info float-end order-edt-btn"
                                data-deliveryid={deliveryId} onClick={onClickHandler}>
                            <svg className="bi" fill="currentColor"><use xlinkHref={pencilImg}/></svg>
                        </button>
                    </li>
                );
            }
        };


        //Add the Add Product Button
        let addProductStyle = {display: 'block'};
        if(currentOrder.hasOwnProperty('productsCost')) {
            addProductStyle = {display: 'none'};
            addExistingOrderButton('product',
                                   frConfig.deliveryDateFromId(currentOrder.deliveryId),
                                   currentOrder.productsCost);
        }

        if (!isOrderReadOnly) {
            ordersByDeliveryBtns.push(
                <li className="list-group-item" id="addProductBtnLi" key="addProductBtnLi" style={addProductStyle}>
                    Add Product Order
                    <button className="btn btn-outline-info float-end order-edt-btn" onClick={onAddOrder}>
                        +
                    </button>
                </li>
            );
        }

        //Add the Add Donation Button
        let addDonationStyle = {display: 'block'};
        if(currentOrder.hasOwnProperty('donation')) {
            addDonationStyle = {display: 'none'};
            addExistingOrderButton('donation', 'Donation', currentOrder.donation);

        }

        if (!isOrderReadOnly) {
            ordersByDeliveryBtns.push(
                <li className="list-group-item" id="addDonationBtnLi" key="addDonationBtnLi" style={addDonationStyle}>
                    Add Donations
                    <button className="btn btn-outline-info float-end order-edt-btn" onClick={onAddDonation}>
                        +
                    </button>
                </li>
            );
        }

        return ordersByDeliveryBtns;
    };
    const ordersByDeliveryBtns = populateOrdersList();

    // Handle disclaimer when neighborhood changes
    const onHoodChange = (evt: any)=>{
        if (evt.currentTarget.value.startsWith("Out of Area")) {
            //console.log(`Hood is: ${evt.currentTarget.value}`);
            jQuery(`#outOfHoodDisclaimer`).show();
        } else {
            jQuery(`#outOfHoodDisclaimer`).hide();
        }
    };



    // Neighborhoods list creation
    let isUsingCustomNeighborhood = false;
    const hoods=[];
    for (const hood of frConfig.neighborhoods()) {
        if (currentOrder.neighborhood && (hood !== currentOrder.neighborhood)) {
            isUsingCustomNeighborhood=true;
        }

        hoods.push(<option key={hood}>{hood}</option>);
    }
    const orderOwners=[];
    for (const [uid, fullName] of frConfig.users()) {
        orderOwners.push(<option value={uid} key={uid}>{fullName}</option>);
    }


    const currentNeighborhood = (currentOrder.neighborhood) ?
                                currentOrder.neighborhood :
                                frConfig.neighborhoods()[0];

    // Calulate Current amountDue
    const amountDueStr = USD(calcCurrentOrderCost()).format();
    const amountPaidStr = USD(currency(currentOrder.checkPaid).add(currentOrder.cashPaid)).format();

    const moniedDefaultValue = (formFld: currency|undefined)=>{
        const fld = currency(formFld);
        return (0.00===fld.value) ? undefined : fld.toString();
    };

    const defaultOrderOwner = currentOrder.orderOwner?
                              currentOrder.orderOwner:auth.currentUser().getUsername()

    setFormFields(
        <form className="needs-validation" id="newOrEditOrderForm" noValidate onSubmit={onFormSubmission}>

            <div className="row mb-2 g-2" style={{display: (isAdmin?'block':'none')}}>
                <div className="form-floating col-md-4">
                    <select className="form-control" id="formOrderOwner" defaultValue={defaultOrderOwner}>
                        {orderOwners}
                    </select>
                    <label htmlFor="formOrderOwner">Order Owner</label>
                </div>
            </div>

            <div className="row mb-2 g-2">
                <div className="form-floating col-md-6">
                    <input className="form-control" type="text" autoComplete="fr-new-cust-info" id="formFirstName"
                           placeholder="First Name" required
                           defaultValue={currentOrder.firstName} />
                    <label htmlFor="formFirstName">
                        First Name<small className="form-text text-muted ps-1">*required</small>
                    </label>
                </div>
                <div className="form-floating col-md-6">
                    <input className="form-control" type="text" autoComplete="fr-new-cust-info" id="formLastName"
                           placeholder="Last Name" required
                           defaultValue={currentOrder.lastName}  />
                    <label htmlFor="formLastName">
                        Last Name<small className="form-text text-muted ps-1">*required</small>
                    </label>
                </div>
            </div>

            <div className="row mb-2 g-2">
                <div className="form-floating col-md-6">
                    <input className="form-control" type="text" autoComplete="fr-new-cust-info" id="formAddr1"
                           placeholder="Address 1" required
                           defaultValue={currentOrder.addr1}  />
                    <label htmlFor="formAddr1">
                        Address 1<small className="form-text text-muted ps-1">*required</small>
                    </label>
                </div>
                <div className="form-floating col-md-6">
                    <input className="form-control" type="text" autoComplete="fr-new-cust-info" id="formAddr2"
                           placeholder="Address 2"
                           defaultValue={currentOrder.addr2}/>
                    <label htmlFor="formAddr2">Address 2</label>
                </div>
            </div>

            <div className="row mb-2 g-2">
                <div className="form-floating col-md-4">
                    <select className="form-control" id="formNeighborhood" onChange={onHoodChange}
                            defaultValue={currentNeighborhood}>
                        {hoods}
                    </select>
                    <label htmlFor="formNeighborhood">
                        Neighborhood<small className="form-text text-muted ps-1">*required</small>
                    </label>
                    <small id="outOfHoodDisclaimer" style={{display: 'none'}}>
                        <i className="bi bi-exclamation-triangle-fill pe-1"></i>
                        You are responsible for delivery of all out of area orders
                        <i className="bi bi-exclamation-triangle-fill ps-1"></i>
                    </small>
                </div>
                <div className="form-floating col-md-4" id="formPhoneFloatDiv">
                    <input className="form-control" type="tel" autoComplete="fr-new-cust-info" id="formPhone"
                           pattern={`[0-9]{3}[-|.]{0,1}[0-9]{3}[-|.]{0,1}[0-9]{4}`}
                           placeholder="Phone" required
                           defaultValue={currentOrder.phone}  />
                    <label htmlFor="formPhone">
                        Phone
                        <small className="form-text text-muted ps-1">(xxx-xxx-xxxx)</small>
                        <small className="form-text text-muted ps-1">*required</small>
                    </label>
                </div>
                <div className="form-floating col-md-4">
                    <input className="form-control" type="text" autoComplete="fr-new-cust-info" id="formEmail"
                           placeholder="Email"
                           defaultValue={currentOrder.email}/>
                    <label htmlFor="formEmail">Email</label>
                </div>
            </div>

            <div className="row mb-2 g-2">
                <div className="form-floating col-md-12">
                    <textarea className="form-control" id="formSpecialInstructions" rows="2"
                              defaultValue={currentOrder.specialInstructions}>
                    </textarea>
                    <label htmlFor="formSpecialInstructions">Special Delivery Instructions</label>
                </div>
            </div>

            <div className="row mb-2 my-2 g-2" style={{display: "contents", border: "0"}}>
                <div className="form-control" id="productList">
                    <ul className="list-group">
                        {ordersByDeliveryBtns}
                    </ul>
                </div>
                <div className="invalid-feedback">
                    *Either a donation or a product order is required
                </div>
            </div>

            <div className="mb-2 my-2 g-2 form-control" style={{display: "flex"}} id="totalsFormRow">
                <div className="row">
                    <div className="col-md-2">
                        <label className="form-check-label" htmlFor="formCollectLater">Collect Later</label>
                        <div className="form-check form-switch">
                            <input className="form-check-input" type="checkbox" id="formCollectLater"
                                   defaultChecked={currentOrder.doCollectMoneyLater}  />
                        </div>
                    </div>
                    <div className="col-md-3">
                        <label htmlFor="formCashPaid">Total Cash Amount</label>
                        <div className="input-group">
                            <div className="input-group-prepend">
                                <span className="input-group-text">$</span>
                            </div>
                            <input className="form-control" type="number" min="0" step="any"
                                   autoComplete="fr-new-cust-info"
                                   id="formCashPaid" placeholder="0.00"
                                   onInput={recalculateTotalPaid} onKeyPress={onCurrencyFieldKeyPress}
                                   defaultValue={moniedDefaultValue(currentOrder.cashPaid)}/>
                        </div>
                    </div>
                    <div className="col-md-3">
                        <label htmlFor="formCheckPaid">Total Check Amount</label>
                        <div className="input-group">
                            <div className="input-group-prepend">
                                <span className="input-group-text">$</span>
                            </div>
                            <input className="form-control" type="number" min="0" step="any"
                                   autoComplete="fr-new-cust-info"
                                   id="formCheckPaid" placeholder="0.00"
                                   onInput={recalculateTotalPaid} onKeyPress={onCurrencyFieldKeyPress}
                                   defaultValue={moniedDefaultValue(currentOrder.checkPaid)}/>
                        </div>
                    </div>
                    <div className="col-md-4">
                        <label htmlFor="formCheckNumbers">Enter Check Numbers</label>
                        <input className="form-control" autoComplete="fr-new-cust-info"
                               id="formCheckNumbers" placeholder="Enter Check #s"
                               onKeyPress={onCheckNumsKeyPress}
                               defaultValue={currentOrder.checkNums}/>
                    </div>


                    <div className="row mb-2 my-2 g-2">
                        <span className="col-md-6">
                            Total Due: <div id="orderAmountDue" style={{display: "inline"}}>{amountDueStr}</div>
                        </span>
                        <span className="col-md-6 g-2" aria-describedby="orderAmountPaidHelp">
                            Total Paid: <div id="orderAmountPaid" style={{display: "inline"}}>{amountPaidStr}</div>
                        </span>
                    </div>
                </div>
            </div>
            <div className="invalid-feedback">
                *Must match total due or the check amount field is populated but there are no check numbers
            </div>


            <div className="pt-4">
                <button type="button" className="btn btn-primary" id="formOrderCancel" onClick={onDiscardOrder}>
                    Cancel
                </button>
                { !isOrderReadOnly &&
                  <button type="submit" className="btn btn-primary float-end" id="formOrderSubmit">
                      <span className="spinner-border spinner-border-sm me-1" role="status"
                      aria-hidden="true" id="formOrderSubmitSpinner" style={{display: "none"}} />
                      Submit
                  </button>
                }
            </div>

        </form>
    );

}



export default (params: any)=>{

    // Calculate Initial total due and amount paid from orders at page load time
    const [formFields, setFormFields] = useState();
    useEffect(() => {

        const onAsyncView = async ()=>{
            const isAdmin = await auth.isCurrentUserAdmin();
            const isOrderReadOnly = (order)=>{
                if (isAdmin) { return false; } //Admins can always edit an order
                return params?.location?.state?.isOrderReadOnly || order?.meta?.isReadOnly;
            };

            const loadOrder = async ()=>{
                const dbOrderId = params?.location?.state?.editOrderId;
                const dbOrderOwner = params?.location?.state?.editOrderOwner;
                if (dbOrderId) {
                    const order: Order|undefined = await orderDb.getOrderFromId(dbOrderId, dbOrderOwner);
                    console.log(`Returned Order: ${JSON.stringify(order)}`);

                    if (order) {
                        const isReadOnly = isOrderReadOnly(order);
                        orderDb.setActiveOrder(order, isReadOnly);
                        await populateForm(order, setFormFields, isAdmin, isReadOnly);
                    } else {
                        alert(`Order: ${dbOrderId} could not be retrieved`);
                        navigate('/reports/');
                    }
                } else {
                    console.error(new Error("Failed to retrieve active order"));
                    navigate('/');
                }
            };


            const order = orderDb.getActiveOrder();
            if (undefined===order) {
                await loadOrder();
            } else {
                await populateForm(order, setFormFields, isAdmin, isOrderReadOnly(order));
            }
        };

        onAsyncView()
            .then()
            .catch((err)=>{
                if ('Invalid Session' === err.message) {
                    navigate('/');
                } else {
                    const errStr = `Failed getting order: ${JSON.stringify(err)}`;
                    console.error(errStr);
                    //alert(errStr);
                    throw err;
                }
            });

    }, [])

    return (
        <>
            <div className="col-xs-1 justify-content-center">
                <div className="card">
                    <div className="card-body">
                        <h5 className="card-title">Customer Information</h5>
                        {formFields}
                    </div>
                </div>
            </div>
        </>
    );
}
