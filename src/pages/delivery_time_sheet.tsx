import React, { useState, useEffect }from "react"
import { navigate } from "gatsby"
import auth from "../js/auth"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config";
import bootstrapIconSprite from "bootstrap-icons/bootstrap-icons.svg";

const exportImg = bootstrapIconSprite + "#cloud-download";

const isTupleEq = (t1, t2) => {
    if (!t1 && !t2) { return true; }
    if (!t1 || !t2) { return false; }
    return(t1[0]===t2[0] && t1[1]===t2[1]); 
}

const pad = (val)=>{return (val<10) ? '0' + val : val };

const testDb1 = [
    { deliveryId: 1, uid: "aidanh", timeIn: "01:00", timeOut: "18:00", timeTotal: "17:00" }
];

const testDb2 = [];

////////////////////////////////////////////////////
//
const getSavedTimeCards = async (deliveryId: number) => {
    if (1===deliveryId) { return testDb1; }
    if (2===deliveryId) { return testDb2; }
    return [];
};

////////////////////////////////////////////////////
//
const saveTimeCard = async (dbRec: any) => {
    const updateDb = (db, dbRec)=>{
        for (const rec of db) {
            if (rec.uid === dbRec.uid) {
                rec.timeIn = dbRec.timeIn;
                rec.timeOut = dbRec.timeOut;
                rec.timeTotal = dbRec.timeTotal;
                return;
            }
        }
        // Not Found so push
        db.push(dbRec);

    };
    
    
    if (1===dbRec.deliveryId) {
        console.log(`SavingToDB1: ${JSON.stringify(dbRec)}`);
        updateDb(testDb1, dbRec);
    }
    if (2===dbRec.deliveryId) {
        console.log(`SavingToDB2: ${JSON.stringify(dbRec)}`);
        updateDb(testDb2, dbRec);
    }
    return [];
};

////////////////////////////////////////////////////
//
const clearTimeCard = async (dbRec: any) => {
    const updateDb = (db, dbRec)=>{
        const idx = db.findIndex((rec)=>{
            return dbRec.uid===rec.uid;
        });
        if (-1 !== idx) {
            console.log(`Deleting: ${JSON.stringify(dbRec)}`);
            db.splice(idx,1);
        }
    };
    
    
    if (1===dbRec.deliveryId) {
        updateDb(testDb1, dbRec);
    }
    if (2===dbRec.deliveryId) {
        updateDb(testDb2, dbRec);
    }
    return [];
};


////////////////////////////////////////////////////
//
class UserEntry {
    uid: string;
    deliveryId: number;
    timeIn: string;
    timeOut: string;
    calcTime: string;
    newTimeIn: string;
    newTimeOut: string;
    newCalcTime: string;
    
    ////////////////////////////////////////////////////
    //
    constructor(params: any) {
        this.deliveryId = parseInt(params.deliveryId);
        // console.log(`deliveryId: ${this.deliveryId}`);
        if (params.hasOwnProperty("timeIn")) {
            this.timeIn = params.timeIn;
            // console.log(`TimeIn: ${this.timeIn}`);
        }
        if (params.hasOwnProperty("timeOut")) {
            this.timeOut = params.timeOut;
            // console.log(`TimeOut: ${this.timeOut}`);
        }
        if (params.hasOwnProperty("timeTotal")) {
            this.calcTime = params.timeTotal;
            // console.log(`calcTime: ${this.calcTime}`);
        }
        if (params.hasOwnProperty("uid")) {
            this.uid = params.uid;
            // console.log(`uid: ${this.uid}`);
        }
    }

    ////////////////////////////////////////////////////
    //
    setNewTime(tmIn: string, tmOut: string, calcTime: string): boolean {
        this.newTimeIn = tmIn;
        this.newTimeOut = tmOut;
        this.newCalcTime = calcTime;

        // Return true if needs saving
        // console.log(`NCT: ${this.newCalcTime} === CT ${this.calcTime}`);
        return this.newCalcTime !== this.calcTime;
    }

    ////////////////////////////////////////////////////
    //
    async save() {
        this.timeIn = this.newTimeIn;
        this.timeOut = this.newTimeOut;
        this.calcTime = this.newCalcTime;

        const rec = {
            deliveryId: this.deliveryId,
            uid: this.uid,
            timeIn: this.timeIn,
            timeOut: this.timeOut,
            timeTotal: this.calcTime,
        };

        if (!this.calcTime || 0===this.calcTime.length || '00:00'===this.calcTime) {
            await clearTimeCard(rec);
        } else {
            await saveTimeCard(rec);
        }
    }
}
type Uid = string;
const timeCardDb: Map<Uid, UserEntry> = new Map();

////////////////////////////////////////////////////
//
const setNewTime = (uid: string,
                    deliveryId: number,
                    timeInComp: [number, number],
                    timeOutComp: [number, number]): [string, boolean] =>
{
    if (0===timeInComp[0] &&
        0===timeInComp[1] &&
        isTupleEq(timeInComp, timeOutComp))
    {
        const isDirty = timeCardDb.has(uid) ?
                        timeCardDb.get(uid).setNewTime(undefined, undefined, undefined) : false;

        return ['00:00', isDirty];
    }

    
    const dtIn = new Date(Date.UTC(0, 0, 0, timeInComp[0],timeInComp[1], 0));
    const dtOut = new Date(Date.UTC(0, 0, 0, timeOutComp[0],timeOutComp[1], 0));

    // console.log(`In: ${dtIn.getTime()}    Out: ${dtOut.getTime()}`);
    const diffMs = dtOut - dtIn;
    if (0>=diffMs) {
        return ["INV", false];
    } else {
        const h = Math.floor(diffMs / (1000*60*60));
        const m = Math.round((diffMs - ((1000*60*60) * h)) / (1000*60));
        if (isNaN(h) || isNaN(m)) {
            return ["INV", false];
        } else {
            if (!timeCardDb.has(uid)) {
                timeCardDb.set(uid, new UserEntry({deliveryId: deliveryId, uid: uid}));
            }
            const calcTime = `${pad(h)}:${pad(m)}`;
            const isDirty = timeCardDb.get(uid).setNewTime(
                `${pad(timeInComp[0])}:${pad(timeInComp[1])}`,
                `${pad(timeOutComp[0])}:${pad(timeOutComp[1])}`,
                calcTime);
            //console.log(`IsDirty ${isDirty}`);
            return [calcTime, isDirty];
        }
    }
}

/////////////////////////////////////////////
//
const onTimeChange = (evt: any)=>{
    const rowElm = evt.currentTarget.parentNode.parentNode.parentNode;
    const timeInVal = rowElm.querySelector(".time-in").value;
    const timeOutVal = rowElm.querySelector(".time-out").value;
    const timeCalcElm = rowElm.querySelector(".time-calc");
    const btnElm = rowElm.querySelector(".btn");
    const deliveryId = document.getElementById("timeSheetSelectDeliveryDate").value;

    let updateStat: [string, boolean] = ["", false];
    if (timeInVal && timeOutVal) {
        let timeInComp = timeInVal.split(":");
        let timeOutComp = timeOutVal.split(":");
        if (2==timeInComp.length && 2==timeOutComp.length) {
            updateStat = setNewTime(
                rowElm.dataset.uid,
                deliveryId,
                [parseInt(timeInComp[0]),parseInt(timeInComp[1])],
                [parseInt(timeOutComp[0]),parseInt(timeOutComp[1])]
            );
        }
    } else {
        updateStat = setNewTime(rowElm.dataset.uid, deliveryId, [0,0], [0,0]);
    }

    const displayVal = updateStat[0];
    const isDirty = updateStat[1];
    timeCalcElm.innerHTML = displayVal;
    if ("INV"===displayVal) {
        timeCalcElm.classList.add("is-invalid");
    } else {
        timeCalcElm.classList.remove("is-invalid");
    }

    if (isDirty) {
        btnElm.classList.remove("invisible");
    } else {
        btnElm.classList.add("invisible");
    }
};

/////////////////////////////////////////////
//
const onSave = async (evt: any)=>{
    const rowElm = evt.currentTarget.parentNode.parentNode;
    const btnElm = evt.currentTarget;
    const spinnyElm = btnElm.querySelector(".spinner-border");
    spinnyElm.style.display = "inline-block";
    btnElm.disabled = true;

    const uid = rowElm.dataset.uid;
    const userName = rowElm.dataset.uname;
    
    console.log(`Saving time for: ${uid}`);
    if (!timeCardDb.has(uid)) {
        console.error(`User ${uid} not found in timeCardDb`);
        alert(`Failed to save time card for: ${userName}}`);
        return;
    }

    await timeCardDb.get(uid).save();
    btnElm.disabled = false;
    spinnyElm.style.display = "none";
    btnElm.classList.add("invisible");
};

//Need it as a global to be picked up by onDeliveryChange
let frConfig = undefined;

////////////////////////////////////////////////////
//
export default function deliveryTimeSheet() {

    
    const [userEntries, setUserEntries] = useState();
    const [deliveryDateOpts, setDeliveryDateOpts] = useState();
    ////////////////////////////////////////////////////
    //
    useEffect(() => {
        const onLoadComponent = async ()=>{
            const [isValidSession, session] = await auth.getSession();
            if (!isValidSession) {
                // If no active user go to login screen
                navigate('/');
                return;
            }

            try {
                frConfig = getFundraiserConfig();
            } catch(err) {
                console.error(`Failed loading fundraiser config going to main page`);
                navigate('/');
                return;
            }

            const deliveryDates = [];
            for (const [frDeliveryId, frDeliveryLabel] of frConfig.deliveryDates()) {
                if ('donation'===frDeliveryId) { continue; }
                deliveryDates.push(
                    <option value={frDeliveryId} key={frDeliveryId}>{frDeliveryLabel}</option>
                );
            }
            setDeliveryDateOpts(deliveryDates);
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

    ////////////////////////////////////////////////////
    //
    const onDeliveryChange = async (evt) =>{
        const currentDeliveryId = parseInt(evt.currentTarget.value);
        const btnElm = document.querySelector(".reports-view-setting-btn");
        btnElm.classList.remove("invisible");

        console.log(`Current Selected DeliveryId: ${currentDeliveryId}`);

        const entries = [];
        for (const [uid, userName] of frConfig.users({doFilterOutAdmins: true})) {
            //console.log(`UserInfo ${JSON.stringify(userInfo)}`);
            const timeInId = `timeInId-${uid}`;
            const timeOutId = `timeOutId-${uid}`;
            const timeCalcId = `timeCalcId-${uid}`;

            // Populate Fields
            entries.push(<li key={uid} className="list-group-item">
                <div className="row" data-uid={uid} data-uname={userName}>
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
                            <div id={timeCalcId} className="form-control time-calc">00:00</div>
                            <label htmlFor={timeCalcId}>Total Time</label>
                        </div>
                    </div>
                    <div className="col">
				                <button type="button" className="btn btn-primary invisible" onClick={onSave}>
                            <span className="spinner-border spinner-border-sm me-1" role="status"
							                    aria-hidden="true" style={{display: "none"}} />
                            Save
                        </button>
                    </div>
                </div>
            </li>);
        }

        setUserEntries(entries);

        timeCardDb.clear();
        jQuery(".time-in").val('');
        jQuery(".time-out").val('');
        jQuery(".time-calc").text('00:00');
        const timeCards = await getSavedTimeCards(currentDeliveryId);
        for (const timecard of timeCards) {
            timeCardDb.set(timecard.uid, new UserEntry(timecard));
            const rowElm = document.querySelector(`.row[data-uid="${timecard.uid}"]`);
            if (!rowElm) {
                alert(`TimeCard DB contains UID: ${timecard.uid} however not is User DB`);
                continue;
            }
            rowElm.querySelector(".time-in").value = timecard.timeIn;
            rowElm.querySelector(".time-out").value = timecard.timeOut;
            rowElm.querySelector(".time-calc").innerHTML = timecard.timeTotal;
        }
    };


    ////////////////////////////////////////////////////
    //
    const onDownloadTimecardsClick = async ()=> {
        const currentDeliveryId = parseInt(document.getElementById("timeSheetSelectDeliveryDate").value);
        const deliveryDate = frConfig.deliveryDateFromId(currentDeliveryId);
        const timeCardFileName = `TimeCardReport-${deliveryDate}.csv`;
        console.log(`Generating Report for: ${timeCardFileName}`);

        if (0===timeCardDb.size) {
            alert("No entries found to download");
            return;
        }
        //Get Data
        let csvData = [];
        for (const [uid, entry] of timeCardDb.entries()) {
            const csvRow = [];
            csvRow.push(uid);
            csvRow.push(frConfig.getUserNameFromId(uid));
            csvRow.push(entry.timeIn);
            csvRow.push(entry.timeOut);
            csvRow.push(entry.calcTime);
            csvData.push(csvRow);
        }
        const headers = ["Id", "FullName", "TimeIn", "TimeOut", "TotalTime"];
        
        csvData = Papa.unparse({
	          "fields": headers,
	          "data": csvData,
        });

        const hiddenElement = document.createElement('a');
		    const blob = new Blob([csvData], { type: 'text/plain;charset=utf-8' });
		    hiddenElement.href = URL.createObjectURL(blob);
        hiddenElement.target = '_blank';
        hiddenElement.download = timeCardFileName;
        hiddenElement.click();
    };


    
    ////////////////////////////////////////////////////
    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <div className="card">
                <div className="card-body">
                    <h5 className="card-title">Delivery Timesheet</h5>
						        <div className="row mb-2">
                        <span>Select Delivery Date
							              <select defaultValue={-1} className="ms-1"
                                    id="timeSheetSelectDeliveryDate" onChange={onDeliveryChange}>
                                <option disabled value={-1}>Select Date</option>
								                {deliveryDateOpts}
							              </select>
                            <button type="button" className="btn reports-view-setting-btn invisible ms-3"
                                    onClick={onDownloadTimecardsClick} data-bs-toggle="tooltip" title="Download Timecards">
                                <svg className="bi" fill="currentColor">
                                    <use xlinkHref={exportImg}/>
                                </svg>
                            </button>
                        </span>
						        </div>
                    <ul className="list-group" id="timeSheet">
                        {userEntries}
                    </ul>
                </div>
            </div>
        </div>
    );
}
