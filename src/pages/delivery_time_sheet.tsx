import React, { useState, useEffect }from "react"
import { navigate } from "gatsby"
import auth from "../js/auth"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config";



export default function deliveryTimeSheet() {

    const [userEntries, setUserEntries] = useState();
    const [deliveryDateOpts, setDeliveryDateOpts] = useState();
    useEffect(() => {
        const onLoadComponent = async ()=>{
            const [isValidSession, session] = await auth.getSession();
            if (!isValidSession) {
                // If no active user go to login screen
                navigate('/');
                return;
            }

            const frConfig = getFundraiserConfig();

            const deliveryDates = [];
            for (const [frDeliveryId, frDeliveryLabel] of frConfig.deliveryDates()) {
                if ('donation'===frDeliveryId) { continue; }
                deliveryDates.push(
                    <option value={frDeliveryId} key={frDeliveryId}>{frDeliveryLabel}</option>
                );
            }
            setDeliveryDateOpts(deliveryDates);

            const entries = [];
            for (const [uid, userName] of frConfig.users({doFilterOutAdmins: true})) {
                //console.log(`UserInfo ${JSON.stringify(userInfo)}`);
                const timeInId = `timeInId-${uid}`;
                const timeOutId = `timeOutId-${uid}`;
                const timeCalcId = `timeCalcId-${uid}`;

                const onTimeChange = (evt: any)=>{
                    const rowElm = evt.currentTarget.parentNode.parentNode.parentNode;
                    const timeInVal = rowElm.querySelector(".time-in").value;
                    const timeOutVal = rowElm.querySelector(".time-out").value;
                    const timeCalcElm = rowElm.querySelector(".time-calc");
                    if (timeInVal && timeOutVal) {
                        let timeInComp = timeInVal.split(":");
                        let timeOutComp = timeOutVal.split(":");
                        if (2==timeInComp.length && 2==timeOutComp.length) {
                            const dtIn = new Date(Date.UTC(0, 0, 0, parseInt(timeInComp[0]),parseInt(timeInComp[1]), 0));
                            const dtOut = new Date(Date.UTC(0, 0, 0, parseInt(timeOutComp[0]),parseInt(timeOutComp[1]), 0));

                            console.log(`In: ${dtIn.getTime()}    Out: ${dtOut.getTime()}`);
                            const diffMs = dtOut - dtIn;
                            if (0>=diffMs) {
                                timeCalcElm.innerHTML = "INV";
                            } else {
                                const h = Math.floor(diffMs / (1000*60*60));
                                const m = Math.round((diffMs - ((1000*60*60) * h)) / (1000*60));
                                if (isNaN(h) || isNaN(m)) {
                                    timeCalcElm.innerHTML = "INV";
                                } else {
                                    const pad = (val)=>{return (val<10) ? '0' + val : val };
                                    timeCalcElm.innerHTML = `${pad(h)}:${pad(m)}`;
                                }
                            }
                        }
                    } else {
                        timeCalcElm.innerHTML = `00:00`;
                    }

                };

                entries.push(<li key={uid} className="list-group-item">
                    <div className="row">
                        <div className="col">
                            {userName}
                        </div>
                        <div className="col">
                            <div className="form-floating">
                                <input data-clocklet="format: HH:mm;" onInput={onTimeChange}
                                       className="form-control time-in" id={timeInId} />
                                <label htmlFor={timeInId}>Time In</label>
                            </div>
                        </div>
                        <div className="col">
                            <div className="form-floating">
                                <input data-clocklet="format: HH:mm;" onInput={onTimeChange}
                                       className="form-control time-out" id={timeOutId} />
                                <label htmlFor={timeOutId}>Time Out</label>
                            </div>
                        </div>
                        <div className="col">
                            <div className="form-floating">
                                <div  id={timeCalcId} className="form-control time-calc">00:00</div>
                                <label htmlFor={timeCalcId}>Total Time</label>
                            </div>
                        </div>
                        <div className="col">
                            <button type="button" className="btn btn-outline-primary">Save</button>
                        </div>
                    </div>
                </li>);
            }
            setUserEntries(entries);

        };

        onLoadComponent()
            .then(()=>{})
            .catch((err)=>{
                if ('Invalid Session'===err.message) {
                    navigate('/');
                    return;
                } else {
                    console.error(err);
                }
            });

    }, []);


    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <div className="card">
                <div className="card-body">
                    <h5 className="card-title">Delivery Timesheet</h5>
						        <div className="row mb-2">
                        <span>Select Delivery Date
							              <select className="ms-1" id="timeSheetSelectDeliveryDate">
								                {deliveryDateOpts}
							              </select>
                        </span>
						        </div>
                    <ul className="list-group">
                        {userEntries}
                    </ul>
                    <button type="button" className="btn btn-outline-primary">Save All</button>
                </div>
            </div>
        </div>
    );
}
