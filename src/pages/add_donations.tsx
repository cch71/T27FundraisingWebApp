import React, { useState, useEffect }from "react"
import {orderDb, Order, OrdersForDeliveryDate} from "../js/ordersdb"
import { navigate } from "gatsby"
import currency from "currency.js"


export default function addDonation() {

    const [formFields, setFormFields] = useState();
    useEffect(() => {
        const currentOrder: Order = orderDb.getActiveOrder();

        if (!currentOrder) {
            navigate('/');
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

            const amountDue = currency((document.getElementById('formDonationAmount') as HTMLInputElement).value);

            if (amountDue) {
                const donationOrder: DeliverableOrderIf = {
                    amountDue: amountDue,
                    kind: 'donation'
                };
                currentOrder.orderByDelivery['donation'] = donationOrder;
            } else {
                delete currentOrder.orderByDelivery['donation'];
            }

            navigate('/order_step_1/');
        }

        let donationAmt = undefined;
        let currentDonation = currentOrder.orderByDelivery['donation'];
        if (undefined!==currentDonation) {
            donationAmt=currency(currentDonation.amountDue).toString();
        }

        setFormFields(
            <form onSubmit={onFormSubmission}>

                <div className="form-group row col-sm-12">
                    <label htmlFor="formDonationAmount">Donation</label>
                    <div className="input-group mb-3">
                        <div className="input-group-prepend">
                            <span className="input-group-text">$</span>
                        </div>
                        <input type="number" className="form-control" id="formDonationAmount"
                               defaultValue={donationAmt}
                               placeholder="0.00"
                               onInput={doesSubmitGetEnabled}/>
                    </div>
                </div>

                <button type="button" className="btn btn-primary my-2" onClick={onCancelItem}>
                    Cancel
                </button>
                <button type="submit" className="btn btn-primary my-2 float-right"
                        disabled={undefined===donationAmt} id="formDonationSubmit">
                    Add                            
                </button>
            </form>
        );
    }, []);


    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <div className="card">
                <div className="card-body">
                    <h5 className="card-title">Add Donation</h5>
                    {formFields}
                </div>
            </div>
        </div>
    );
    
}
