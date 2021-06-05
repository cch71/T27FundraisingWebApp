import React, { useState, useEffect } from "react";
import { navigate } from "gatsby";
import awsConfig from "../config";
import auth from "../js/auth";
import {orderDb, Order, OrdersForDeliveryDate} from "../js/ordersdb";
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config";
import currency from "currency.js";
import {onCurrencyFieldKeyPress, moneyRoundFromDouble} from "../js/utils";
import CurrencyWidget from "../components/currency_widget";

import bootstrapIconSprite from "bootstrap-icons/bootstrap-icons.svg";
const exportImg = bootstrapIconSprite + "#cloud-download";

let frConfig;

//const exportImg = bootstrapIconSprite + "#cloud-download";

const pad = (val)=>{return (val<10) ? '0' + val : val };
const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });

////////////////////////////////////////////////////
//
const savedVals = {
    spreadingTotal: USD(0),
    bagsSpread: 0,
    bagsSold: 0,
    salesFromBags: USD(0),
    profitsFromBags: USD(0),
    perBagCost: 0.0,
    deliveryMinutes: 0,
    totalDonations: USD(0),
    bankDeposited: USD(0), // USD("$55,045.40"),
    mulchCost: USD(0), //USD("$22,319.70"),
    allocationPerBagAdjustmentRatio: 0.0, // Percentage to adjust from sales price
    allocationPerDeliveryMinutes: 0.0,
    allocationsForMulchBagSales: USD(0),
}
let dbData = undefined;
let perUserReport = undefined;

////////////////////////////////////////////////////
//
const getSpreadingPrice = (): currency => {
    for (const product of frConfig.products()) {
        if ("spreading" !== product.id) {
            continue;
        }
        return currency(product.unitPrice);
    }
    return currency(0.00);
}

////////////////////////////////////////////////////
//
const getMulchBagUnitPrice = (): [currency, currency, [any]] => {
    for (const product of frConfig.products()) {
        if ("bags" !== product.id) {
            continue;
        }

        let minUnitPrice = product.unitPrice;
        for (const priceBreak of product.priceBreaks) {
            if (minUnitPrice > priceBreak.unitPrice) {
                minUnitPrice = priceBreak.unitPrice;
            }
        }

        return [currency(product.unitPrice), currency(minUnitPrice), product.priceBreaks];
    }
    return [currency(0.00), currency(0.00), []];
}


////////////////////////////////////////////////////
//
const makeTimeCardsCall = async (body: any)=>{
    const userId = auth.currentUser().getUsername();
    const authToken = await auth.getAuthToken();

    const paramStr = JSON.stringify(body);

    //console.log(`OrderDB Query Parms: {}`);
    const resp = await fetch(awsConfig.api.invokeUrl + '/timecards', {
        method: 'post',
        headers: {
            'Content-Type': 'application/json',
            Authorization: authToken
        },
        body: paramStr
    });

    if (!resp.ok) { // if HTTP-status is 200-299
        const errRespBody = await resp.text();
        throw new Error(`Timecard API Failed Error: ${resp.status}  ${errRespBody}`);
    }

    return await resp.json();
};

////////////////////////////////////////////////////
//
const getSavedTimeCards = async () => {
    try {
        console.log("Gettting Timecard Values");
        return await makeTimeCardsCall({
            cmd: 'query',
            payload: {
                deliveryId: 'all'
            }
        });
    } catch(error) {
        console.error(error);
        alert(`Get TimeCards for delivery all Failed: ${error}`);
    }

    return undefined;
};

////////////////////////////////////////////////////
//
const getDeliveryMinutes = async () => {
    let totalMins = 0;
    const timeCards = await getSavedTimeCards();
    //console.log(`Saved Time Cards:\n${JSON.stringify(timeCards, null, '\t')}`);
    let totalMinsPerWorker = {}
    for (const timeCard of timeCards) {
        if (timeCard.timeTotal) {
            let times = timeCard.timeTotal.split(":");
            const worker_mins = parseInt(times[1]) + (60 * parseInt(times[0]));
            totalMins += worker_mins;
            if (!totalMinsPerWorker.hasOwnProperty(timeCard.uid)) {
                totalMinsPerWorker[timeCard.uid] = 0;
            }
            totalMinsPerWorker[timeCard.uid] += worker_mins;
        }
    }
    return [totalMins, totalMinsPerWorker];
};

////////////////////////////////////////////////////
//
const gatherDbData = async (frConfig: FundraiserConfig) => {

    const [maxOrigUnitPrice,minOrigUnitPrice, priceBreaks] = getMulchBagUnitPrice();
    const fieldNames = ['orderOwner', 'spreaders', 'products.spreading', 'products.bags', 'productsCost', 'deliveryId', 'totalAmt']
    const promises = [ orderDb.query({fields: fieldNames, orderOwner: 'any'}), getDeliveryMinutes()];
    const [orders, [deliveryMins, deliveryMinsPerWorker]] = await Promise.all(promises);

    return {
        orders: orders,
        deliveryMinutes: deliveryMins,
        deliveryMinutesPerWorker: deliveryMinsPerWorker,
        pricePerBagToSpread: getSpreadingPrice(frConfig),
        originalBagCosts: {
            maxUnitPrice: maxOrigUnitPrice,
            minUnitPrice: minOrigUnitPrice,
            priceBreaks: priceBreaks
        }
    };
};
////////////////////////////////////////////////////
//
// Helper function for handling bags sold allocation amount
const calcBagSales = (bagsSold)=>{
    let rate = dbData.originalBagCosts.maxUnitPrice;
    // Handle Price product price breaks if any
    for (const priceBreak of dbData.originalBagCosts.priceBreaks) {
        const unitsNeeded = priceBreak.gt;
        if (bagsSold > unitsNeeded) {
            rate = currency(priceBreak.unitPrice);
        }
    }
    return rate.multiply(bagsSold);
};

////////////////////////////////////////////////////
//
export default function fundsRelease() {

    const [isLoading, setIsLoading] = useState(false);
    const [bankDeposited, setBankDeposited] = useState();
    const [mulchCost, setMulchCost] = useState();
    const [totalDonated, setTotalDonated] = useState();
    const [scoutTotalCollected, setScoutTotalCollected] = useState();
    const [spreadingTotal, setSpreadingTotal] = useState();
    const [bagsSpread, setBagsSpread] = useState();
    const [mulchSalesGross, setMulchSalesGross] = useState();

    const [troopPercentage, setTroopPercentage] = useState();
    const [scoutPercentage, setScoutPercentage] = useState();
    const [scoutDeliveryPercentage, setScoutDeliveryPercentage] = useState();
    const [scoutSellingPercentage, setScoutSellingPercentage] = useState();

    const [bagsSold, setBagsSold] = useState();
    const [bagsTotalSales, setBagsTotalSales] = useState();
    const [perBagAvgEarnings, setPerBagAvgEarnings] = useState();
    const [deliveryMinutes, setDeliveryMinutes] = useState();
    const [deliveryEarnings, setDeliveryEarnings] = useState();


    ////////////////////////////////////////////////////
    //
    const getAllocationSummary = ()=>{
        return {
            bankDeposited: bankDeposited,
            mulchCost: mulchCost,
            bagsSold: bagsSold,
            bagsTotalSales: bagsTotalSales,
            bagsSpread: bagsSpread,
            spreadingTotal: spreadingTotal,
            totalDonated: totalDonated,
            totalCollected: scoutTotalCollected,
            mulchSalesGross: mulchSalesGross,
            troopMinAllocation: troopPercentage,
            scoutsMaxAllocation: scoutPercentage,
            scoutsBadSalesAllocation: scoutSellingPercentage,
            perBagAvgEarnings: perBagAvgEarnings,
            scoutDeliveryAllocation: scoutDeliveryPercentage,
            totalDeliveryMinutes: deliveryMinutes,
            deliveryAllocation: deliveryEarnings,
        };
    };


    ////////////////////////////////////////////////////
    //
    useEffect(() => {
        const onLoadComponent = async ()=>{
            setIsLoading(true);
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

            dbData = await gatherDbData(frConfig);
            const orders = dbData.orders;
            savedVals.deliveryMinutes = dbData.deliveryMinutes;

            //console.log(`Orders:\n${JSON.stringify(orders, null, '\t')}`);
            let scoutCollected = currency(0.0);
            for (const order of orders) {
                const totAmt = currency(order.totalAmt);
                scoutCollected = scoutCollected.add(totAmt);

                if (order.products?.spreading) {
                    savedVals.bagsSpread += parseInt(order.products.spreading);
                }

                if (order.products?.bags) {
                    savedVals.bagsSold += parseInt(order.products.bags);
                    savedVals.salesFromBags = savedVals.salesFromBags.add(calcBagSales(order.products.bags));
                }

                // Donations
                if (!order.deliveryId) {
                    savedVals.totalDonations = savedVals.totalDonations.add(totAmt);
                }
                // Also Donations see bug #70
                if (order.deliveryId && order.productsCost) {
                    const prodCost = currency(order.productsCost);
                    if (totAmt.intValue > prodCost.intValue) { // This is a donation
                        savedVals.totalDonations = savedVals.totalDonations.add(
                            totAmt.subtract(prodCost)
                        );
                    }
                }
            }

            savedVals.spreadingTotal = USD(dbData.pricePerBagToSpread.multiply(savedVals.bagsSpread));
            setBankDeposited(savedVals.bankDeposited.format());
            setMulchCost(savedVals.mulchCost.format());
            setScoutTotalCollected(USD(scoutCollected).format());
            setSpreadingTotal(savedVals.spreadingTotal.format());
            setBagsSold(savedVals.bagsSold);
            setBagsSpread(savedVals.bagsSpread);
            setDeliveryMinutes(savedVals.deliveryMinutes);
            setTotalDonated(USD(savedVals.totalDonations).format())
            setBagsTotalSales(savedVals.salesFromBags.format());

        };

        onLoadComponent()
            .then(()=>{
                setIsLoading(false);
                onAllocationFormInputsChange();
            })
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
    const onDownloadSummary = async ()=> {
        const allocationSummary = getAllocationSummary();
        const hiddenElement = document.createElement('a');
        const blob = new Blob([JSON.stringify(allocationSummary, null, 2)], { type: 'application/json' });
        hiddenElement.href = URL.createObjectURL(blob);
        hiddenElement.target = '_blank';
        hiddenElement.download = `AllocationSummary.json`;
        hiddenElement.click();
    };

    ////////////////////////////////////////////////////
    //
    const onDownloadReport = async (evt: any)=> {
        const data = JSON.parse(evt.currentTarget.dataset.reportfields);
        const headers = JSON.parse(evt.currentTarget.dataset.reportheaders);

        const csvData = Papa.unparse({
            "fields": headers,
            "data": data,
        });

        const hiddenElement = document.createElement('a');
        const blob = new Blob([csvData], { type: 'text/plain;charset=utf-8' });
        hiddenElement.href = URL.createObjectURL(blob);
        hiddenElement.target = '_blank';
        hiddenElement.download = `FundsReleaseReport.csv`;
        hiddenElement.click();
    };


    ////////////////////////////////////////////////////
    //
    const onAllocationFormInputsChange = (event: any)=>{
        const bankDeposited = document.getElementById('formBankDeposited').value;
        const mulchCost = document.getElementById('formMulchCost').value;
        //TODO Save savedVals bankDeposited and mulchCost
        console.log(`BD: ${bankDeposited}, MS: ${mulchCost}, SP: ${savedVals.spreadingTotal}`);

        const totalGross = USD(bankDeposited)
            .subtract(USD(savedVals.spreadingTotal))
            .subtract(USD(mulchCost))
            .subtract(savedVals.totalDonations);

        setMulchSalesGross(totalGross.format());

        setTroopPercentage(totalGross.multiply(.20).format());

        const scoutPercentage = totalGross.multiply(.80);
        setScoutPercentage(scoutPercentage.format());
        const [sellingPercentage, deliveryPercentage] = scoutPercentage.distribute(2);
        setScoutDeliveryPercentage(deliveryPercentage.format());
        savedVals.allocationsForMulchBagSales = sellingPercentage;
        setScoutSellingPercentage(savedVals.allocationsForMulchBagSales.format());

        const perBagAvgCanEarn = USD(sellingPercentage/savedVals.bagsSold);
        setPerBagAvgEarnings(perBagAvgCanEarn.format());

        savedVals.perBagCost = USD(mulchCost).value / savedVals.bagsSold;
        savedVals.profitsFromBags = savedVals.salesFromBags.subtract(USD(mulchCost));

        //console.log(`perBagCost: ${savedVals.perBagCost}`);
        //console.log(`profitsFromBags: ${savedVals.profitsFromBags}`);

        savedVals.allocationPerDeliveryMinutes = deliveryPercentage/savedVals.deliveryMinutes;
        setDeliveryEarnings(USD(savedVals.allocationPerDeliveryMinutes).format());

        const isValid = 0.0 < currency(bankDeposited).value && 0.0 < 0.0 < currency(mulchCost).value;
        let btnGenReport = document.getElementById('generateReportsBtn');
        if (isValid) {
            btnGenReport.disabled = false;
        } else {
            btnGenReport.disabled = true;
        }
    };


    ////////////////////////////////////////////////////
    //
    const onReleaseFundsFormSubmission = async (event: any) => {
        ///////////////////////////////
        //
        const make_req = async (body: any) => {
            const params = {
                method: 'post',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': await auth.getAuthToken(),
                }
            };

            if (body) {
                params['body'] = JSON.stringify(body);
            }

            //console.log(`Making Request: ${JSON.stringify(body, null, '\t')}`);

            const resp = await fetch(`${awsConfig.api.invokeUrl}/users`, params);
            if (!resp.ok) {
                const bodyStr = await resp.text()
                throw Error(`HTTP Resp Error: ${resp.status} ${bodyStr}`);
            }
            return await resp.json();
        }

        const btn = document.getElementById('releaseFundsBtn');
        btn.disabled = true;
        const submitSpinny = document.getElementById('formReleaseFundsSpinner');
        submitSpinny.style.display = "inline-block";
        try {
            event.preventDefault();
            event.stopPropagation();
            console.log("Releasing Funds");

            const allocationSummary = getAllocationSummary();

            const reportToSave = {
                perUserSummary: perUserReport,
                allcationSummary: allocationSummary,
                areFundsReleased: true,
            };

            //console.log(`Allocation Summary: ${JSON.stringify(reportToSave, null, '\t')}`);

            await make_req({cmd: 'UPDATE_LEADERBOARD', summary: reportToSave});
        } catch(error) {
            console.error(error);
            alert(`Failed Saving Allocation Report: ${error}`);
        }

        submitSpinny.style.display = "none";
        btn.disabled = false;

        /* const hiddenElement = document.createElement('a');
         * const blob = new Blob([JSON.stringify(reportToSave, null, 2)], { type: 'application/json' });
         * hiddenElement.href = URL.createObjectURL(blob);
         * hiddenElement.target = '_blank';
         * hiddenElement.download = `LeaderboardSummary.json`;
         * hiddenElement.click();
         */
    }


    const [reportCard, setReportCard] = useState();

    ////////////////////////////////////////////////////
    //
    const onAllocationFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();
        console.log("Generating Report");
        perUserReport = {};

        // Helper function for handling spreaders
        // TODO:  Sum all the amounts and then do truncing at the end. don't use distribute
        const recordSpreaders = (bagsToSpread, spreaders) => {
            const numSpreaders = spreaders.length;
            const allocationDist = (dbData.pricePerBagToSpread.value * bagsToSpread) / numSpreaders;
            for (let idx=0; idx<numSpreaders; ++idx) {
                const uid = spreaders[idx];
                if (!perUserReport.hasOwnProperty(uid)) { perUserReport[uid] = {}; }
                if (!perUserReport[uid].hasOwnProperty('rawAllocationFromBagsSpread')) {
                    perUserReport[uid]['rawAllocationFromBagsSpread'] = 0.0
                }
                perUserReport[uid]['rawAllocationFromBagsSpread'] += allocationDist;
            }
        };

        // Go through timecards for delivery and calculate delivery costs
        for (const [uid, mins] of Object.entries(dbData.deliveryMinutesPerWorker)) {
            if (!perUserReport.hasOwnProperty(uid)) { perUserReport[uid] = {}; }
            perUserReport[uid]['deliveryMins'] = mins;
            perUserReport[uid]['allocationsFromDelivery'] =
                USD(moneyRoundFromDouble(savedVals.allocationPerDeliveryMinutes * mins));
        }


        // Go through orders to get the rest of the information
        for (const order of dbData.orders) {
            const totAmt = currency(order.totalAmt);
            const uid = order.orderOwner;
            if (!perUserReport.hasOwnProperty(uid)) { perUserReport[uid] = {}; }
            if (!perUserReport[uid].hasOwnProperty('totalAmtCollected')) {
                perUserReport[uid]['totalAmtCollected'] = USD(0);
            }
            perUserReport[uid]['totalAmtCollected'] =
                perUserReport[uid]['totalAmtCollected'].add(totAmt);

            // Add a donation
            if (!order.deliveryId) {
                if (!perUserReport[uid].hasOwnProperty('donations')) {
                    perUserReport[uid]['donations'] = USD(0);
                }
                perUserReport[uid]['donations'] = perUserReport[uid]['donations'].add(totAmt);
            }

            // Also Donations see bug #70
            if (order.deliveryId && order.productsCost) {
                const prodCost = currency(order.productsCost);
                if (totAmt.intValue > prodCost.intValue) { // This is a donation
                    if (!perUserReport[uid].hasOwnProperty('donations')) {
                        perUserReport[uid]['donations'] = USD(0);
                    }
                    perUserReport[uid]['donations'] = perUserReport[uid]['donations'].add(
                        totAmt.subtract(prodCost)
                    );
                }
            }

            // Calculate spreading/spreaders
            if (order.products?.spreading) {
                if (!perUserReport[uid].hasOwnProperty('numBagsSpreadSold')) {
                    perUserReport[uid]['numBagsSpreadSold'] = 0;
                }
                perUserReport[uid]['numBagsSpreadSold'] += order.products.spreading;
                if (order.spreaders) {
                    recordSpreaders(order.products.spreading, order.spreaders);
                }
            }

            // Calculate Mulch Sales
            if (order.products?.bags) {
                if (!perUserReport[uid].hasOwnProperty('numBagsSold')) {
                    perUserReport[uid]['numBagsSold'] = 0;
                    perUserReport[uid]['allocationFromBagsSold'] = USD(0);
                    perUserReport[uid]['totalCollectedForBags'] = USD(0);
                }

                // To get allocation based on break down, get percentage of sales and use that percentage
                //  to get that percentage from allocation
                perUserReport[uid]['totalCollectedForBags'] =  
                    perUserReport[uid]['totalCollectedForBags'].add(calcBagSales(order.products.bags));

                perUserReport[uid]['numBagsSold'] += order.products.bags;

            }
        }

        for (const [uid, report] of Object.entries(perUserReport)) {
            if (report.hasOwnProperty('numBagsSold')) {
                const troopCostForBags = savedVals.perBagCost * report.numBagsSold;
                const profitFromBags = report.totalCollectedForBags - troopCostForBags;
                const percentageOfSales = profitFromBags / savedVals.profitsFromBags.value;
                const allocatedAmt =
                    USD(moneyRoundFromDouble(percentageOfSales * savedVals.allocationsForMulchBagSales));
                perUserReport[uid]['allocationFromBagsSold'] = allocatedAmt;
            }

            if (report.hasOwnProperty('rawAllocationFromBagsSpread')) {
                report['allocationFromBagsSpread'] = 
                    USD(moneyRoundFromDouble(report.rawAllocationFromBagsSpread));
            }

        }

        ////////////////////////////////////////////////////
        //
        const getTotalAllocations = (report)=>{
            let totalAllocation = USD(0);
            if (report.hasOwnProperty('allocationFromBagsSold')) {
                totalAllocation = totalAllocation.add(report['allocationFromBagsSold']);
            }
            if (report.hasOwnProperty('allocationFromBagsSpread')) {
                totalAllocation = totalAllocation.add(report['allocationFromBagsSpread']);
            }
            if (report.hasOwnProperty('allocationsFromDelivery')) {
                totalAllocation = totalAllocation.add(report['allocationsFromDelivery']);
            }
            if (report.hasOwnProperty('donations')) {
                totalAllocation = totalAllocation.add(report['donations']);
            }

            return totalAllocation;
        };

        const perScoutReportDataHeaders = [ "UserId",
                                            "FullName",
                                            "BagsSold",
                                            "BagsSpread",
                                            "DeliveryMinutes",
                                            "Donations",
                                            "AllocFrmBagsSold",
                                            "AllocFrmBagsSpread",
                                            "AllocFrmDelivery",
                                            "AllocTotal" ];
        const perScoutReportDataFields = [];
        let totalDonations = USD(0);
        let allocationFromBagsSold = USD(0);
        let allocationFromBagsSpread = USD(0);
        let allocationsFromDelivery = USD(0);
        let allocationsTotal = USD(0);

        for (const [uid, report] of Object.entries(perUserReport)) {
            if (report.hasOwnProperty('donations')) {
                totalDonations = totalDonations.add(report['donations']);
            }
            if (report.hasOwnProperty('allocationFromBagsSold')) {
                allocationFromBagsSold = allocationFromBagsSold.add(report['allocationFromBagsSold']);
            }
            if (report.hasOwnProperty('allocationFromBagsSpread')) {
                allocationFromBagsSpread = allocationFromBagsSpread.add(report['allocationFromBagsSpread']);
            }
            if (report.hasOwnProperty('allocationsFromDelivery')) {
                allocationsFromDelivery = allocationsFromDelivery.add(report['allocationsFromDelivery']);
            }
            report['allocationTotal'] = getTotalAllocations(report);
            allocationsTotal = allocationsTotal.add(report['allocationTotal'])

            perScoutReportDataFields.push([
                uid,
                frConfig.getUserNameFromId(uid),
                report.hasOwnProperty('numBagsSold')?report['numBagsSold'] : '',
                report.hasOwnProperty('numBagsSpreadSold')?report['numBagsSpreadSold'] : '',
                report.hasOwnProperty('deliveryMins')?report['deliveryMins'] : '',
                report.hasOwnProperty('donations')?report['donations'].format() : '',
                report.hasOwnProperty('allocationFromBagsSold')?report['allocationFromBagsSold'].format() : '',
                report.hasOwnProperty('allocationFromBagsSpread')?report['allocationFromBagsSpread'].format() : '',
                report.hasOwnProperty('allocationsFromDelivery')?report['allocationsFromDelivery'].format() : '',
                report['allocationTotal'].format()
            ]);
        }

        perScoutReportDataFields.push([
            '',
            'Scout Total Allocations',
            savedVals.bagsSold,
            savedVals.bagsSpread,
            savedVals.deliveryMinutes,
            totalDonations.format(), // I could pull all the below from global data but they wanted cross check
            allocationFromBagsSold.format(),
            allocationFromBagsSpread.format(),
            allocationsFromDelivery.format(),
            allocationsTotal.format()
        ]);

        const perScoutReport = [];
        // Skip totals since we do that different but we want it included in downloaded csv
        for (let idx=0; idx < perScoutReportDataFields.length-1; ++idx) {
            const field = perScoutReportDataFields[idx];
            perScoutReport.push(
                <tr key={field[0]}>
                    <td>{field[1]}</td>
                    <td>{field[2]}</td>
                    <td>{field[3]}</td>
                    <td>{field[4]}</td>
                    <td>{field[5]}</td>
                    <td>{field[6]}</td>
                    <td>{field[7]}</td>
                    <td>{field[8]}</td>
                    <td>{field[9]}</td>
                </tr>
            );
        }

        setReportCard(
            <div className="col-md-9">
                <div className="card">
                    <h5 className="card-header justify-content-center text-center">
                        Allocation Report
                        <button type="button" className="btn reports-view-setting-btn ms-3"
                                onClick={onDownloadReport} data-bs-toggle="tooltip"
                                data-reportfields={JSON.stringify(perScoutReportDataFields)}
                                data-reportheaders={JSON.stringify(perScoutReportDataHeaders)}
                                title="Download Report">
                            <svg className="bi" fill="currentColor">
                                <use xlinkHref={exportImg}/>
                            </svg>
                        </button>
                    </h5>
                    <div className="card-body">
                        <form onSubmit={onReleaseFundsFormSubmission}>
                            <div className="table-responsive-xxl" id="fundsReleaseTables">
                                <table className="table table-striped">
                                    <thead>
                                        <tr>
                                            <th scope="col">Name</th>
                                            <th scope="col"># Bags Sold</th>
                                            <th scope="col"># Bags to Spread Sold</th>
                                            <th scope="col"># Delivery Minutes</th>
                                            <th scope="col">$ Donations</th>
                                            <th scope="col">$ Allocations from Bags Sold</th>
                                            <th scope="col">$ Allocations from Spreading</th>
                                            <th scope="col">$ Allocations from Delivery</th>
                                            <th scope="col">$ Total Allocations</th>
                                        </tr>
                                        <tr style={{"backgroundColor": "DarkSeaGreen"}}>
                                            <td>Scout Alloc Totals</td>
                                            <td>{savedVals.bagsSold}</td>
                                            <td>{savedVals.bagsSpread}</td>
                                            <td>{savedVals.deliveryMinutes}</td>
                                            <td>{totalDonations.format()}</td>
                                            <td>{allocationFromBagsSold.format()}</td>
                                            <td>{allocationFromBagsSpread.format()}</td>
                                            <td>{allocationsFromDelivery.format()}</td>
                                            <td>{allocationsTotal.format()}</td>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {perScoutReport}
                                    </tbody>
                                </table>
                            </div>
                            <button type="submit" className="btn btn-primary my-2 float-end"
                                    id="releaseFundsBtn"
                                    data-bs-toggle="tooltip"
                                    title="Release Report to Scouts">
                                <span className="spinner-border spinner-border-sm me-1" role="status"
                                      aria-hidden="true" id="formReleaseFundsSpinner" style={{display: "none"}} />
                                Save and Release Funds
                            </button>
                        </form>
                    </div>
                </div>
            </div>
        );
    }


    ////////////////////////////////////////////////////
    return (<>
        {isLoading ? (
            <div id="notReadyView" className='col-xs-1 d-flex justify-content-center' >
                <div className="spinner-border" role="status">
                    <span className="visually-hidden">Loading...</span>
                </div>
            </div>
        ) : (
            <>
                <div className="col-xs-1 d-flex justify-content-center">
                    <h4>Funds Release Page</h4>
                </div>
                <div className="releaseFundsCards">
                    <div className="row">

                        <div className="col">

                            <div className="card" style={{'maxWidth': '30rem'}}>
                                <h5 className="card-header justify-content-center text-center">
                                    Allocation Calculations
                                    <button type="button" className="btn reports-view-setting-btn ms-3"
                                            onClick={onDownloadSummary} data-bs-toggle="tooltip"
                                            title="Download Summary">
                                        <svg className="bi" fill="currentColor">
                                            <use xlinkHref={exportImg}/>
                                        </svg>
                                    </button>
                                </h5>
                                <div className="card-body">
                                    <form onSubmit={onAllocationFormSubmission}>
                                        <div className="row mb-2">
                                            <CurrencyWidget id="formBankDeposited"
                                                            defaultValue={bankDeposited}
                                                            label="Amount Deposited in Bank"
                                                            onInput={onAllocationFormInputsChange}
                                            />
                                        </div>
                                        <div className="row mb-2">
                                            <CurrencyWidget id="formMulchCost"
                                                            defaultValue={mulchCost}
                                                            label="Amount Paid for Mulch"
                                                            onInput={onAllocationFormInputsChange}
                                            />
                                        </div>

                                        <div className="table-responsive" id="fundsReleaseTables">
                                            <table className="table table-striped caption-top">
                                                <caption>Sales</caption>
                                                <thead>
                                                    <tr>
                                                        <th scope="col"></th>
                                                        <th scope="col">Num Sold</th>
                                                        <th scope="col">Sales</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    <tr>
                                                        <td>Bags of Mulch</td>
                                                        <td>{bagsSold}</td>
                                                        <td>{bagsTotalSales}</td>
                                                    </tr>
                                                    <tr>
                                                        <td>Spreading Jobs</td>
                                                        <td>{bagsSpread}</td>
                                                        <td>{spreadingTotal}</td>
                                                    </tr>
                                                    <tr>
                                                        <td>Donations</td>
                                                        <td></td>
                                                        <td>{totalDonated}</td>
                                                    </tr>
                                                </tbody>
                                                <tfoot>
                                                    <tr>
                                                        <td>Total Collected</td>
                                                        <td></td>
                                                        <td>{scoutTotalCollected}</td>
                                                    </tr>
                                                </tfoot>
                                            </table>

                                            <table className="table table-striped table-responsive caption-top">
                                                <caption>Allocations</caption>
                                                <tbody>
                                                    <tr>
                                                        <td>Gross Profits</td>
                                                        <td>{mulchSalesGross}</td>
                                                    </tr>
                                                    <tr>
                                                        <td>Min Allocations to Troop (est)</td>
                                                        <td>{troopPercentage}</td>
                                                    </tr>
                                                    <tr>
                                                        <td>Max Allocations to Scouts (est)</td>
                                                        <td>{scoutPercentage}</td>
                                                    </tr>
                                                    <tr>
                                                        <td colSpan="4">
                                                            <table className="table table-striped caption-top mb-0">
                                                                <caption>Scout Allocations</caption>
                                                                <tbody>
                                                                    <tr>
                                                                        <td>For Mulch Bag Sales (est)</td>
                                                                        <td>{scoutSellingPercentage}</td>
                                                                    </tr>
                                                                    <tr>
                                                                        <td>Avg Allocation per Bag</td>
                                                                        <td>{perBagAvgEarnings}</td>
                                                                    </tr>
                                                                    <tr>
                                                                        <td>For Delivery (est)</td>
                                                                        <td>{scoutDeliveryPercentage}</td>
                                                                    </tr>
                                                                    <tr>
                                                                        <td>Total Delivery Minutes</td>
                                                                        <td>{deliveryMinutes}</td>
                                                                    </tr>
                                                                    <tr>
                                                                        <td>Allocation Per Delivery Minute</td>
                                                                        <td>{deliveryEarnings}</td>
                                                                    </tr>
                                                                </tbody>
                                                            </table>
                                                        </td>
                                                    </tr>
                                                </tbody>
                                                <tfoot>
                                                </tfoot>
                                            </table>
                                        </div>

                                        <button type="submit" className="btn btn-primary my-2 float-end"
                                                id="generateReportsBtn"
                                                data-bs-toggle="tooltip"
                                                title="Generate Data">
                                            Generate Data
                                        </button>
                                    </form>
                                </div>
                            </div> {/* End of card */}
                        </div>
                        {reportCard}
                    </div>
                </div>
            </>
        )}
    </>);
}
