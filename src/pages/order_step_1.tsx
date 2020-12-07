import React, { useState, useEffect } from "react"
import { Router, Link } from '@reach/router';
import NavBar from "../components/navbar"
import {orderDb, Order} from "../js/ordersdb"
//import OrderItem from "../components/order_item" //TODO: Rename DeliveryOrderSummary
import { navigate } from "gatsby"
import currency from "currency.js"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"


const USD = (value) => currency(value, { symbol: "$", precision: 2 });

const AddProduct = React.lazy(() => import('./add_donations'));
const AddDonation = React.lazy(() => import('./add_products_order'));
const SignOn = React.lazy(() => import('./signon'));

const LazyComponent = ({ Component, ...props }) => (
    <React.Suspense fallback={'<p>Loading...</p>'}>
        <Component {...props} />
    </React.Suspense>
);


const populateForm = (currentOrder: Order, setFormFields: any): any =>{

    const fundraiserConfig: FundraiserConfig = getFundraiserConfig();
    if (!fundraiserConfig) {
        alert("Failed to load fundraiser config");
        return(<div/>);
    }

    // Save off current order values
    const saveCurrentOrder = ()=>{
        //Required
        currentOrder.firstName = (document.getElementById('formFirstName') as HTMLInputElement).value;
        currentOrder.lastName = (document.getElementById('formLastName') as HTMLInputElement).value;
        currentOrder.phone = (document.getElementById('formPhone') as HTMLInputElement).value;
        currentOrder.addr1 = (document.getElementById('formAddr1') as HTMLInputElement).value;
        currentOrder.neighborhood = (document.getElementById('formNeighborhood') as HTMLSelectElement).value;
        

        currentOrder.email = (document.getElementById('formEmail') as HTMLInputElement).value;
        currentOrder.addr2 = (document.getElementById('formAddr2') as HTMLInputElement).value;
        /* currentOrder.city = (document.getElementById('formCity') as HTMLInputElement).value;
         * currentOrder.state = (document.getElementById('formState') as HTMLInputElement).value;
         * currentOrder.zip = (document.getElementById('formZip') as HTMLInputElement).value;
        */
         currentOrder.specialInstructions =
             (document.getElementById('formSpecialInstructions') as HTMLInputElement).value;
        currentOrder.checkNumbers = (document.getElementById('formCheckNumbers') as HTMLInputElement).value;
        currentOrder.cashPaid =
            currency((document.getElementById('formCashPaid') as HTMLInputElement).value);
        currentOrder.checkPaid =
            currency((document.getElementById('formCheckPaid') as HTMLInputElement).value);
        currentOrder.amountTotal = currentOrder.cashPaid.add(currentOrder.checkPaid);
        console.log(`Current Order ${JSON.stringify(currentOrder, null, 2)}`);
    }
    
    // Handle Form Submission
    const onFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();

        //console.log(`Submitting Active Order`);
        saveCurrentOrder();
        if (doesSubmitGetEnabled()) {
            orderDb.submitActiveOrder().then(()=>{
                navigate('/');
            }).catch((err:any)=>{
                if ('Invalid Session'===err) {
                    navigate('/signon/')
                } else {
                    const errStr = `Failed submitting order: ${JSON.stringify(err)}`;
                    console.log(errStr);
                    alert(errStr);
                    throw err;
                }
            });
        } else {
            console.error(`Form submitted but shouldn't have been`);
        }
    }

    // Add New Product Order
    const onAddOrder = (event: any)=>{
        event.preventDefault();
        event.stopPropagation();

        saveCurrentOrder();

        const btn = event.currentTarget;

        let pageState = {state: {}};
        if (btn.dataset && btn.dataset.deliveryid) {
            const deliveryId = btn.dataset.deliveryid;
            console.log(`Add New Fundraising Order for ${deliveryId}`);
            pageState.state['deliveryId'] = deliveryId;
        }

        navigate('/add_products_order/', pageState);
    };

    // Add Donation to order
    const onAddDonation = (event: any)=>{
        event.preventDefault();
        event.stopPropagation();
        console.log(`Adding New Donation`);

        saveCurrentOrder()
        navigate('/add_donations/');
    };

    // Discard the order and go back to home
    const onDiscardOrder = ()=>{
        orderDb.setActiveOrder();
        navigate('/');
    };

    // Check for enabling/disabling submit button
    const doesSubmitGetEnabled = (/*event: any*/)=>{
        const amountDue = currency(document.getElementById('orderAmountDue').innerText);
        const amountPaid = currency(document.getElementById('orderAmountPaid').innerText);
        let isCheckNumGood = true;
        if (0.0!==currency((document.getElementById('formCheckPaid') as HTMLInputElement).value).value) {
            isCheckNumGood = (0!==(document.getElementById('formCheckNumbers') as HTMLInputElement).value.length);
        }
        //console.log(`AD: ${amountDue.value} AP: ${amountPaid.value}`);
        if ( (document.getElementById('formFirstName') as HTMLInputElement).value &&
             (document.getElementById('formLastName') as HTMLInputElement).value &&
             (document.getElementById('formPhone') as HTMLInputElement).value &&
             (document.getElementById('formAddr1') as HTMLInputElement).value &&
             (document.getElementById('formNeighborhood') as HTMLSelectElement).value &&
             (0!==Object.keys(currentOrder.orderByDelivery).length) &&
             (amountDue.value===amountPaid.value) &&
             isCheckNumGood
        )
        {
            (document.getElementById('formOrderSubmit') as HTMLButtonElement).disabled = false;
            return true;
        } else {
            (document.getElementById('formOrderSubmit') as HTMLButtonElement).disabled = true;
            return false;
        }
    };
    
    // Recalculate Total due dynamically based on changes to order status
    const recalculateTotalDue = ()=> {
        let totalDue = currency(0.0);
        for (let deliverable of Object.values(currentOrder.orderByDelivery)) {
            console.log(`Found Order: ${deliverable.amountDue}`);
            totalDue = totalDue.add(deliverable.amountDue);
        }
        const totElm = document.getElementById('orderAmountDue');
        if (null!==totElm) {
            totElm.innerText = `${USD(totalDue).format()}`;
        }
    }

    const recalculateTotalPaid = ()=> {
        const cash = currency((document.getElementById('formCashPaid') as HTMLInputElement).value);
        const checks = currency((document.getElementById('formCheckPaid') as HTMLInputElement).value);
        
        
        const totElm = document.getElementById('orderAmountPaid');
        if (null!==totElm) {
            totElm.innerText = `${USD(cash.add(checks)).format()}`;
            doesSubmitGetEnabled();
        }

    }
    
    // Create delivery status buttons
    const populateOrdersList = (): Array<any> => {
        const ordersByDeliveryBtns = []
        for (const [deliveryId, deliveryLabel] of fundraiserConfig.validDeliveryDates()) {
            const foundOrder = currentOrder.orderByDelivery[deliveryId];
            if (!foundOrder) { continue; }

            const foundTag = `found-${deliveryId}`
            const orderTotalStr = `${deliveryLabel} Amount: ${USD(foundOrder.amountDue).format()} `;

            // On Delete Order Re-enable button
            const onDeleteOrder = (event)=>{
                event.preventDefault();
                event.stopPropagation();

                const deliveryIdToDel = event.currentTarget.dataset.deliveryid;
                
                console.log(`Deleting Order for ${deliveryIdToDel}`);

                delete currentOrder.orderByDelivery[deliveryIdToDel];
                document.getElementById(foundTag).style.display = "none";
                if ('donation' === deliveryIdToDel) {
                    document.getElementById('addDonationBtnLi').style.display = "block";
                } else {
                    document.getElementById('addProductBtnLi').style.display = "block";
                }

                recalculateTotalDue();
                doesSubmitGetEnabled();
            }

            //console.log(`Adding Order Type for DDay: ${deliveryId}`);
            const onClickHandler = ("donation" === deliveryId)? onAddDonation : onAddOrder;

            ordersByDeliveryBtns.push(
                <li className="list-group-item" id={foundTag} key={foundTag}>
                    {orderTotalStr}
                    <button className="btn btn-outline-danger mx-1 float-right order-edt-btn"
                            data-deliveryid={deliveryId} onClick={onDeleteOrder}>
                        <span>&#10005;</span>
                    </button>
                    <button className="btn btn-outline-info float-right order-edt-btn"
                            data-deliveryid={deliveryId} onClick={onClickHandler}>
                        <span>&#9999;</span>
                    </button>
                </li>
            );
        }

        // Figure out visibility
        const orderHasDonation = currentOrder.orderByDelivery.hasOwnProperty('donation');
        let numProdOrders = Object.keys(currentOrder.orderByDelivery).length
        if (orderHasDonation) {
            numProdOrders = numProdOrders - 1;
        }
        const addDonationDisplay = (!orderHasDonation) ?
                                   {display: 'block'}:{display: 'none'};
        
        const addProdDisplay = (0 !== (fundraiserConfig.numDeliveryDates()-numProdOrders)) ?
                               {display: 'block'}:{display: 'none'};

        //Add the Add Product Button
        ordersByDeliveryBtns.push(
            <li className="list-group-item" id="addProductBtnLi" key="addProductBtnLi" style={addProdDisplay}>
                Add Product Order
                <button className="btn btn-outline-info float-right order-edt-btn" onClick={onAddOrder}>
                    <span>+</span>
                </button>
            </li>
        );

        //Add the Add Donation Button
        ordersByDeliveryBtns.push(
            <li className="list-group-item" id="addDonationBtnLi" key="addDonationBtnLi" style={addDonationDisplay}>
                Add Donations
                <button className="btn btn-outline-info float-right order-edt-btn" onClick={onAddDonation}>
                    <span>+</span>
                </button>
            </li>
        );

        return ordersByDeliveryBtns;
    };
    const ordersByDeliveryBtns = populateOrdersList();

    // Neighborhoods list creation
    let isUsingCustomNeighborhood = false;
    const hoods=[];
    for (const hood of fundraiserConfig.neighborhoods()) {
        if (currentOrder.neighborhood && (hood !== currentOrder.neighborhood)) {
            isUsingCustomNeighborhood=true;
        }

        hoods.push(<option key={hood}>{hood}</option>);
    }
    const currentNeighborhood = (currentOrder.neighborhood) ?
                                currentOrder.neighborhood :
                                fundraiserConfig.neighborhoods()[0];
        
    // Calulate Current amountDue
    let newTotalDue = currency(0.0);
    for (let deliverable of Object.values(currentOrder.orderByDelivery)) {
        //console.log(`Found Order To Calc: ${deliverable.amountDue}`);
        newTotalDue = newTotalDue.add(deliverable.amountDue);
    }
    const amountDueStr = USD(newTotalDue).format();
    const amountPaid = currentOrder.checkPaid.add(currentOrder.cashPaid);
    const amountPaidStr = USD(amountPaid).format();
    //console.log(`Amount Due: ${amountDueStr}  Paid: ${amountPaidStr}`);

    const areRequiredCurOrderValuesAlreadyPopulated = (
        currentOrder.firstName &&
        currentOrder.lastName &&
        currentOrder.phone &&
        currentOrder.addr1 &&
        currentOrder.neighborhood &&
        (newTotalDue.value === amountPaid.value));


    // This is if we support city, state, zip
    /* const stateAbbreviations = [
     *     'AL','AK','AS','AZ','AR','CA','CO','CT','DE','DC','FM','FL','GA',
     *     'GU','HI','ID','IL','IN','IA','KS','KY','LA','ME','MH','MD','MA',
     *     'MI','MN','MS','MO','MT','NE','NV','NH','NJ','NM','NY','NC','ND',
     *     'MP','OH','OK','OR','PW','PA','PR','RI','SC','SD','TN','TX','UT',
     *     'VT','VI','VA','WA','WV','WI','WY'
     * ];

     * const states=[];
     * for (let st of stateAbbreviations) {
     *     states.push(<option key={st}>{st}</option>);
     * } */
    /* const wreathInfo = ()=>(
     *     return(
     *         <Form.Row>
     *             <Form.Group as={Col} md="7" controlId="formCity">
     *                 <Form.Label>City</Form.Label>
     *                 <Form.Control required type="text" placeholder="City" defaultValue={currentOrder.city} />
     *             </Form.Group>

     *             <Form.Group as={Col} md="2" controlId="formState">
     *                 <Form.Label>State</Form.Label>
     *                 <Form.Control as="select" defaultValue='TX'>
     *                     {states}
     *                 </Form.Control>
     *             </Form.Group>

     *             <Form.Group as={Col} md="3" controlId="formZip">
     *                 <Form.Label>Zip</Form.Label>
     *                 <Form.Control type="text" placeholder="Zip" defaultValue={currentOrder.zip} />
     *             </Form.Group>
     *         </Form.Row>
     *     );
     * ); */

    const moniedDefaultValue = (fld: currency)=>{
        return (0.00===fld.value) ? undefined : fld.toString();
    };

    setFormFields(
        <form onSubmit={onFormSubmission}>
            
            <div className="form-row">
                <div className="form-group col-md-6">
                    <label htmlFor="formFirstName">First Name</label>
                    <input className="form-control" type="text" autoComplete="fr-new-cust-info"
                           id="formFirstName" placeholder="First Name" defaultValue={currentOrder.firstName}
                           required
                           onInput={doesSubmitGetEnabled}
                           aria-describedby="formFirstNameHelp" />
                    <small id="formFirstNameHelp" className="form-text text-muted">*required</small>
                </div>
                <div className="form-group col-md-6">
                    <label htmlFor="formLastName">Last Name</label>
                    <input className="form-control" type="text" autoComplete="fr-new-cust-info" id="formLastName"
                           required
                           placeholder="Last Name"
                           defaultValue={currentOrder.lastName}
                           onInput={doesSubmitGetEnabled}
                           aria-describedby="formLastNameHelp" />
                    <small id="formLastNameHelp" className="form-text text-muted">*required</small>
                </div>
            </div>

            <div className="form-row">
                <div className="form-group col-md-6">
                    <label htmlFor="formAddr1">Phone</label>
                    <input className="form-control" type="text" autoComplete="fr-new-cust-info" id="formAddr1"
                           required
                           placeholder="Address 1"
                           defaultValue={currentOrder.addr1}
                           onInput={doesSubmitGetEnabled}
                           aria-describedby="formAddr1Help" />
                    <small id="formAddr1Help" className="form-text text-muted">*required</small>
                </div>
                <div className="form-group col-md-6">
                    <label htmlFor="formAddr2">Address 2</label>
                    <input className="form-control" type="text" autoComplete="fr-new-cust-info" id="formAddr2"
                           placeholder="Address 2"
                           defaultValue={currentOrder.addr2}/>
                </div>
            </div>

            <div className="form-row">
                <div className="form-group col-md-4">
                    <label htmlFor="formNeighborhood">Neighborhood</label>
                    <select className="form-control" id="formNeighborhood"
                            aria-describedby="formNeighborhoodHelp" defaultValue={currentNeighborhood}>
                        {hoods}
                    </select>
                    <small id="formNeighborhoodHelp" className="form-text text-muted">*required</small>
                </div>
                <div className="form-group col-md-4">
                    <label htmlFor="formPhone">Phone</label>
                    <input className="form-control" type="text" autoComplete="fr-new-cust-info" id="formPhone"
                           required
                           placeholder="Phone"
                           defaultValue={currentOrder.phone}
                           onInput={doesSubmitGetEnabled}
                           aria-describedby="formPhoneHelp" />
                    <small id="formPhoneHelp" className="form-text text-muted">*required</small>
                </div>
                <div className="form-group col-md-4">
                    <label htmlFor="formEmail">Email</label>
                    <input className="form-control" type="text" autoComplete="fr-new-cust-info" id="formEmail"
                           placeholder="Email"
                           defaultValue={currentOrder.email}/>
                </div>
            </div>

            <div className="form-row">
                <div className="form-group col-md-12">
                    <label htmlFor="formSpecialInstructions">Special Instructions</label>
                    <textarea className="form-control" id="formSpecialInstructions" rows="2"></textarea>
                </div>
            </div>

            <div className="form-row">
                <div className="form-group col-md-4">
                    <label htmlFor="formCashPaid">Total Cash Amount</label>
                    <div className="input-group mb-3">
                        <div className="input-group-prepend">
                            <span className="input-group-text">$</span>
                        </div>
                        <input className="form-control" type="text"  autoComplete="fr-new-cust-info"
                               id="formCashPaid" placeholder="Total Cash Amount"
                               onInput={recalculateTotalPaid}
                               placeholder="0.00"
                               defaultValue={moniedDefaultValue(currentOrder.cashPaid)}/>
                    </div>
                </div>
                <div className="form-group col-md-4">
                    <label htmlFor="formCheckPaid">Total Check Amount</label>
                    <div className="input-group mb-3">
                        <div className="input-group-prepend">
                            <span className="input-group-text">$</span>
                        </div>
                        <input className="form-control" type="text" autoComplete="fr-new-cust-info"
                               id="formCheckPaid" placeholder="Total Check Amount"
                               onInput={recalculateTotalPaid}
                               placeholder="0.00"
                               defaultValue={moniedDefaultValue(currentOrder.checkPaid)}/>
                    </div>
                </div>
                <div className="form-group col-md-4">
                    <label htmlFor="formCheckNumbers">Enter Check Numbers</label>
                    <input className="form-control" type="text" autoComplete="fr-new-cust-info"
                           id="formCheckNumbers" placeholder="Enter Check #s"
                           onInput={doesSubmitGetEnabled}
                           defaultValue={currentOrder.checkNumbers}/>
                </div>
            </div>

            <ul className="list-group">
                {ordersByDeliveryBtns}
            </ul>

            <div className="form-row">
                <span className="form-group col-md-6">
                    Total Due: <div id="orderAmountDue" style={{display: "inline"}}>{amountDueStr}</div>
                </span>
                <span className="form-group col-md-6" aria-describedby="orderAmountPaidHelp">
                    Total Paid: <div id="orderAmountPaid" style={{display: "inline"}}>{amountPaidStr}</div>
                    <small id="orderAmountPaidHelp" className="form-text text-muted">*Must match total due</small>
                </span>
            </div>
            
            <div className="pt-4">
                <button type="button" className="btn btn-primary" onClick={onDiscardOrder}>
                    Cancel
                </button>
                <button type="submit" className="btn btn-primary float-right" id="formOrderSubmit" disabled={!areRequiredCurOrderValuesAlreadyPopulated}>
                    Submit
                </button>
            </div>

        </form>
    );
}



export default (params: any)=>{

    // Calculate Initial total due and amount paid from orders at page load time
    const [formFields, setFormFields] = useState();
    useEffect(() => {
        const order = orderDb.getActiveOrder();
        if (undefined===order) {
            const dbOrderId = params?.location?.state?.editOrderId;
            if (dbOrderId) {
                orderDb.getOrderFromId(dbOrderId).then((order: Order|undefined)=>{
                    console.log(`Returned Order: ${JSON.stringify(order)}`);
                    if (order) {
                        orderDb.setActiveOrder(order);
                        populateForm(order, setFormFields);
                    } else {
                        alert(`Order: ${dbOrderId} could not be retrieved`);
                        navigate('/orders/');
                    }
                }).catch((err: any)=>{
                    if ('Invalid Session'===err) {
                        navigate('/signon/');
                    } else {
                        const errStr = `Failed retrieving order: ${JSON.stringify(err)}`;
                        console.log(errStr);
                        alert(errStr);
                        throw err;
                    }
                });
            } else {
                //alert("Failed to retrieve active order");
                navigate('/');
            }
        } else {
            populateForm(order, setFormFields);
        }
    }, [])    

    return (
        <div>
            <NavBar/>
            <div className="col-xs-1 d-flex justify-content-center">
                <div className="card">
                    <div className="card-body">
                        <h5 className="card-title">Customer Information</h5>
                        {formFields}
                    </div>
                </div>
            </div>

            <Router>
                <LazyComponent Component={AddProduct} path="/add_products_order/" />
                <LazyComponent Component={AddDonation} path="/add_donations/" />
                <LazyComponent Component={SignOn} path="/signon/" />
            </Router>
        </div>
    );
}

