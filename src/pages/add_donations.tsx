import React from "react"
import {orderDb, Order, OrdersForDeliveryDate} from "../js/ordersdb"
import { navigate } from "gatsby"
import currency from "currency.js"


export default function addDonation() {
    const currentOrder: Order = orderDb.getActiveOrder();
    if (!currentOrder) {
        navigate('/');
        return(<div/>);
    }

    const doesSubmitGetEnabled = (event: any)=>{
        if (event.currentTarget.value) {
            (document.getElementById('formDonationSubmit') as HTMLButtonElement).disabled = false;
        } else {
            (document.getElementById('formDonationSubmit') as HTMLButtonElement).disabled = true;
        }
    };

    const onCancelItem = ()=>{
        navigate('/order_step_1/');
    }

    const onFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();

        const donationOrder: DeliverableOrderIf = {
            amountDue: currency((document.getElementById('formDonationAmount') as HTMLInputElement).value),
            kind: 'donation'
        };

        currentOrder.orderByDelivery.set('donation', donationOrder);

        navigate('/order_step_1/');
    }

    let donationAmt = currency(0.0);
    let currentDonation = currentOrder.orderByDelivery.get('donation');
    if (undefined!==currentDonation) {
        donationAmt=currentDonation.amountDue;
    }

    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <div className="card">
                <div className="card-body">
                    <h5 className="card-title">Add Donation</h5>
                    <form onSubmit={onFormSubmission}>

                        <div className="form-group row col-sm-12">
                            <label htmlFor="formDonationAmount">Donation</label>
                            <input type="number" className="form-control" id="formDonationAmount"
                                   defaultValue={donationAmt.toString()}
                                   placeholder="Enter Donation Amount"
                                   onInput={doesSubmitGetEnabled}/>
                        </div>


                        <button type="button" className="btn btn-primary my-2" onClick={onCancelItem}>
                            Back
                        </button>
                        <button type="submit" className="btn btn-primary my-2 float-right"
                                disabled={0.0===donationAmt.value} id="formDonationSubmit">
                            Add                            
                        </button>
                    </form>
                </div>
            </div>
        </div>
    );
    
}
