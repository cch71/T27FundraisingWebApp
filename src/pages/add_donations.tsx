import React, { useState, useEffect }from "react"
import {orderDb, Order, OrdersForDeliveryDate} from "../js/ordersdb"
import { navigate } from "gatsby"
import currency from "currency.js"
import {onCurrencyFieldKeyPress, moneyFloor} from "../js/utils"


export default function addDonation() {

    const [formFields, setFormFields] = useState();
    useEffect(() => {
        const currentOrder: Order = orderDb.getActiveOrder();

        if (!currentOrder) {
            navigate('/');
        }
        const doesSubmitGetEnabled = (event: any)=>{

            const origAmt = (document.getElementById('formDonationAmount') as HTMLInputElement).value;
            const [amt, isChanged] = moneyFloor(origAmt);
            if (isChanged) {
                (document.getElementById('formDonationAmount') as HTMLInputElement).value = amt.toString();
            }

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
                currentOrder['donation'] = amountDue;
            } else {
                delete currentOrder['donation'];
            }

            navigate('/order_step_1/');
        }

        let donationAmt = undefined;
        if (undefined!==currentOrder['donation']) {
            donationAmt=currency(currentOrder['donation']).toString();
        }

        setFormFields(
            <form onSubmit={onFormSubmission}>

                <div className="row col-sm-12">
                    <label htmlFor="formDonationAmount">Donation</label>
                    <div className="input-group mb-3">
                        <div className="input-group-prepend">
                            <span className="input-group-text">$</span>
                        </div>
                        <input type="number" min="0" step="any" className="form-control" id="formDonationAmount"
                               defaultValue={donationAmt}
                               placeholder="0.00"
                               onInput={doesSubmitGetEnabled} onKeyPress={onCurrencyFieldKeyPress}/>
                    </div>
                </div>

                <button type="button" className="btn btn-primary my-2" onClick={onCancelItem}>
                    Cancel
                </button>
                <button type="submit" className="btn btn-primary my-2 float-end"
                        style={{display: ((currentOrder.meta?.isReadOnly)?"none":"block")}}
                        disabled={undefined===donationAmt} id="formDonationSubmit">
                    Submit
                </button>
            </form>
        );
    }, []);


    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <div className="card">
                <div className="card-body">
                    <h5 className="card-title">Donation</h5>
                    {formFields}
                </div>
            </div>
        </div>
    );

}
