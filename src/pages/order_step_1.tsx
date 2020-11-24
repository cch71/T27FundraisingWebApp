import React, { useState, useEffect } from "react"
import NavBar from "../components/navbar"
import {orderDb, NewOrder} from "../js/ordersdb"
import OrderItem from "../components/order_item" //TODO: Rename DeliveryOrderSummary
import { navigate } from "gatsby"
import currency from "currency.js"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"


export default (location: any)=>{
    const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });

    const fundraiserConfig: FundraiserConfig = getFundraiserConfig();
    if (!fundraiserConfig) {
        alert("Failed to load fundraiser config");
        return(<div/>);
    }


    let currentOrder: NewOrder = orderDb.getCurrentOrder();

    const saveCurrentOrder = ()=>{
        currentOrder.firstName = (document.getElementById('formFirstName') as HTMLInputElement).value;
        currentOrder.lastName = (document.getElementById('formLastName') as HTMLInputElement).value;
        currentOrder.email = (document.getElementById('formEmail') as HTMLInputElement).value;
        currentOrder.phone = (document.getElementById('formPhone') as HTMLInputElement).value;
        currentOrder.addr1 = (document.getElementById('formAddr1') as HTMLInputElement).value;
        currentOrder.addr2 = (document.getElementById('formAddr2') as HTMLInputElement).value;
        /* currentOrder.city = (document.getElementById('formCity') as HTMLInputElement).value;
         * currentOrder.state = (document.getElementById('formState') as HTMLInputElement).value;
         * currentOrder.zip = (document.getElementById('formZip') as HTMLInputElement).value;
         * currentOrder.specialInstructions =
         *     (document.getElementById('formSpecialInstructions') as HTMLInputElement).value; */
        currentOrder.cashPaid =
            currency((document.getElementById('formCashPaid') as HTMLInputElement).value);
        currentOrder.checkPaid =
            currency((document.getElementById('formCheckPaid') as HTMLInputElement).value);
        console.log(`Current Order ${JSON.stringify(currentOrder, null, 2)}`);
        orderDb.setCurrentOrder(currentOrder);
    }

    const onFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();

    }

    const onAddOrder = (event: any)=>{
        event.preventDefault();
        event.stopPropagation();

        saveCurrentOrder()

        const btn = event.currentTarget;
        console.log(`Add New Fundraising Order for ${btn.dataset.deliverydate}`);

        navigate('/add_products_order/', {state: {deliveryDate: btn.dataset.deliverydate}});
    };

    const onAddDonation = (event: any)=>{
        event.preventDefault();
        event.stopPropagation();
        console.log(`Adding New Donation`);

        saveCurrentOrder()
        navigate('/add_donations/');
    };

    const doesSubmitGetEnabled = (event: any)=>{
        if (event.currentTarget.value) {
            (document.getElementById('formOrderSubmit') as HTMLButtonElement).disabled = false;
        } else {
            (document.getElementById('formOrderSubmit') as HTMLButtonElement).disabled = true;
        }
    };



    const [initTotalDue, setInitTotalDue] = useState(USD(currency(0.0)).format());
    useEffect(() => {
        let newTotalDue = currency(0.0);
        for (let deliverable of currentOrder.deliverables.values()) {
            console.log(`Found Order: ${deliverable.totalDue}`);
            newTotalDue = newTotalDue.add(deliverable.totalDue);
        }
        /* const totElm = document.getElementById('orderTotalDue');
         * if (null!==totElm) {
         *     totElm.innerText = `Total Due:}`;
         * }
         */
        setInitTotalDue(USD(newTotalDue).format());
    }, [])


    const recalculateTotal = ()=> {
        const totalDue = currency(0.0);
        for (let deliverable of currentOrder.deliverables.values()) {
            console.log(`Found Order: ${deliverable.totalDue}`);
            totalDue = totalDue.add(deliverable.totalDue);
        }
        const totElm = document.getElementById('orderTotalDue');
        if (null!==totElm) {
            totElm.innerText = `Total Due: ${USD(totalDue).format()}`;
        }
    }

    const ordersByDeliveryBtns = []
    for (const deliveryDate of fundraiserConfig.validDeliveryDates()) {
        const onClickHandler = ("donation" === deliveryDate)? onAddDonation : onAddOrder;

        ordersByDeliveryBtns.push(
            <li className="list-group-item" id={deliveryDate} key={deliveryDate}>
                <OrderItem onClick={onClickHandler} deliveryDate={deliveryDate} onDelete={recalculateTotal} />
            </li>
        );
    }

    const hoods=[];
    for (let hood of fundraiserConfig.neighborhoods()) {
        hoods.push(<option key={hood}>{hood}</option>);
    }


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
    

    return (
        <div>
            <NavBar/>
            <div className="col-xs-1 d-flex justify-content-center">
                <div className="card">
                    <div className="card-body">
                        <h5 className="card-title">Customer Information</h5>
                        <form onSubmit={onFormSubmission}>
                            
                            <div className="form-row">
                                <div className="form-group col-md-6">
                                    <label htmlFor="formFirstName">First Name</label>
                                    <input className="form-control" type="text" id="formFirstName"
                                           required
                                           placeholder="First Name"
                                           defaultValue={currentOrder.firstName}
                                           onInput={doesSubmitGetEnabled}
                                           aria-describedby="formFirstNameHelp" />
                                    <small id="formFirstNameHelp" className="form-text text-muted">*required</small>
                                </div>
                                <div className="form-group col-md-6">
                                    <label htmlFor="formLastName">Last Name</label>
                                    <input className="form-control" type="text" id="formLastName"
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
                                    <input className="form-control" type="text" id="formAddr1"
                                           required
                                           placeholder="Address 1"
                                           defaultValue={currentOrder.addr1}
                                           onInput={doesSubmitGetEnabled}
                                           aria-describedby="formAddr1Help" />
                                    <small id="formAddr1Help" className="form-text text-muted">*required</small>
                                </div>
                                <div className="form-group col-md-6">
                                    <label htmlFor="formAddr2">Address 2</label>
                                    <input className="form-control" type="text" id="formAddr2"
                                           placeholder="Address 2"
                                           defaultValue={currentOrder.addr2}/>
                                </div>
                            </div>

                            <div className="form-row">
                                <div className="form-group col-md-4">
                                    <label htmlFor="formNeighborhood">Neighborhood</label>
                                    <select className="form-control" id="formNeighborhood" aria-describedby="formNeighborhoodHelp">
                                        {hoods}
                                    </select>
                                    <small id="formNeighborhoodHelp" className="form-text text-muted">*required</small>
                                </div>
                                <div className="form-group col-md-4">
                                    <label htmlFor="formPhone">Phone</label>
                                    <input className="form-control" type="text" id="formPhone"
                                           required
                                           placeholder="Phone"
                                           defaultValue={currentOrder.phone}
                                           onInput={doesSubmitGetEnabled}
                                           aria-describedby="formPhoneHelp" />
                                    <small id="formPhoneHelp" className="form-text text-muted">*required</small>
                                </div>
                                <div className="form-group col-md-4">
                                    <label htmlFor="formEmail">Email</label>
                                    <input className="form-control" type="text" id="formEmail"
                                           placeholder="Email"
                                           defaultValue={currentOrder.email}/>
                                </div>
                            </div>


                            <div className="form-row">
                                <div className="form-group col-md-4">
                                    <label htmlFor="formCashPaid">Total Cash Amount</label>
                                    <input className="form-control" type="text"
                                           id="formCashPaid" placeholder="Total Cash Amount"
                                           defaultValue={currentOrder.cashPaid.toString()}/>
                                </div>
                                <div className="form-group col-md-4">
                                    <label htmlFor="formCheckPaid">Total Check Amount</label>
                                    <input className="form-control" type="text"
                                           id="formCheckPaid" placeholder="Total Check Amount"
                                           defaultValue={currentOrder.checkPaid.toString()}/>
                                </div>
                                <div className="form-group col-md-4">
                                    Enter Check #S TBD
                                </div>
                            </div>

                            <ul className="list-group">
                                {ordersByDeliveryBtns}
                            </ul>
                            
                            <div>Total Paid: $Calculation TBD</div>
                            <div id="orderTotalDue">Total Due: {initTotalDue}</div>
                            
                            <button type="submit" className="btn btn-primary my-2 float-right" id="formOrderSubmit">
                                Submit
                            </button>
                        </form>
                    </div>
                </div>
            </div>
        </div>
    );


}
