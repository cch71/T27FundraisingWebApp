import React, { useState, useEffect } from "react";
import { navigate } from "gatsby";
import awsConfig from "../config";
import auth from "../js/auth";
import {orderDb, Order, OrdersForDeliveryDate} from "../js/ordersdb";
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config";
import currency from "currency.js";
import {onCurrencyFieldKeyPress, moneyFloorFromDouble} from "../js/utils";
import CurrencyWidget from "../components/currency_widget";

import bootstrapIconSprite from "bootstrap-icons/bootstrap-icons.svg";
const exportImg = bootstrapIconSprite + "#cloud-download";

import rawDbData from "./RawFundsReleaseData.json"
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
    deliveryMinutes: 0,
    totalDonations: USD(0),
    bankDeposited: USD("$54,979.90"),
    mulchCost: USD("$22,319.70"),
    allocationPerBagAdjustmentRatio: 0.0, // Percentage to adjust from sales price
    allocationPerDeliveryMinutes: 0.0,
    allocationsForMulchBagSales: USD(0),
}
let dbData = undefined;

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

    rawDbData.pricePerBagToSpread = currency(rawDbData.pricePerBagToSpread);
    const [maxOrigUnitPrice,minOrigUnitPrice, priceBreaks] = getMulchBagUnitPrice();
    rawDbData['originalBagCosts'] = {
        maxUnitPrice: maxOrigUnitPrice,
        minUnitPrice: minOrigUnitPrice,
        priceBreaks: priceBreaks
    };
    return rawDbData;
    /* const fieldNames = ['orderOwner', 'spreaders', 'products.spreading', 'products.bags', 'productsCost', 'deliveryId', 'totalAmt']
       const promises = [ orderDb.query({fields: fieldNames, orderOwner: 'any'}), getDeliveryMinutes()];
       const [orders, [deliveryMins, deliveryMinsPerWorker]] = await Promise.all(promises);

       return {
       orders: orders,
       deliveryMinutes: deliveryMins,
       deliveryMinutesPerWorker: deliveryMinsPerWorker,
       pricePerBagToSpread: getSpreadingPrice(frConfig)
       }; */
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
    const onDownloadRawData = async ()=> {
        const hiddenElement = document.createElement('a');
        const blob = new Blob([JSON.stringify(dbData, null, 2)], { type: 'application/json' });
        hiddenElement.href = URL.createObjectURL(blob);
        hiddenElement.target = '_blank';
        hiddenElement.download = `RawFundsReleaseData.json`;
        hiddenElement.click();
    };

    ////////////////////////////////////////////////////
    //
    const onDownloadReport = async ()=> {
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

        savedVals.allocationPerDeliveryMinutes = deliveryPercentage/savedVals.deliveryMinutes;
        setDeliveryEarnings(USD(savedVals.allocationPerDeliveryMinutes).format());

        const isValid = 0.0 < currency(bankDeposited).value && 0.0 < 0.0 < currency(mulchCost).value;
        let btnGenReport = document.getElementById('generateReportsBtn');
        if (isValid) {
            btnGenReport.classList.remove("invisible");
        } else {
            btnGenReport.classList.add("invisible");
        }
    };


    ////////////////////////////////////////////////////
    //
    const onReleaseFundsFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();
        console.log("Releasing Funds");



    }


    const [reportCard, setReportCard] = useState();

    ////////////////////////////////////////////////////
    //
    const onAllocationFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();
        console.log("Generating Report");
        const perUserReport = {};

        // Helper function for handling spreaders
        const recordSpreaders = (bagsToSpread, spreaders) => {
            const numSpreaders = spreaders.length;
            const allocationDist =
                USD(moneyFloorFromDouble(dbData.pricePerBagToSpread.value * bagsToSpread)).distribute(numSpreaders);
            for (let idx=0; idx<numSpreaders; ++idx) {
                const uid = spreaders[idx];
                if (!perUserReport.hasOwnProperty(uid)) { perUserReport[uid] = {}; }
                if (!perUserReport[uid].hasOwnProperty('allocationFromBagsSpread')) {
                    perUserReport[uid]['allocationFromBagsSpread'] = USD(0);
                }
                perUserReport[uid]['allocationFromBagsSpread'] =
                    perUserReport[uid]['allocationFromBagsSpread'].add(allocationDist[idx]);
            }
        };

        // Go through timecards for delivery and calculate delivery costs
        for (const [uid, mins] of Object.entries(dbData.deliveryMinutesPerWorker)) {
            if (!perUserReport.hasOwnProperty(uid)) { perUserReport[uid] = {}; }
            perUserReport[uid]['deliveryMins'] = mins;
            perUserReport[uid]['allocationsFromDelivery'] =
                USD(moneyFloorFromDouble(savedVals.allocationPerDeliveryMinutes * mins));
        }


        // Go through orders to get the rest of the information
        for (const order of dbData.orders) {
            const totAmt = currency(order.totalAmt);
            const uid = order.orderOwner;
            if (!perUserReport.hasOwnProperty(uid)) { perUserReport[uid] = {}; }

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
                }

                // To get allocation based on break down, get percentage of sales and use that percentage
                //  to get that percentage from allocation
                const orderOrigBagSales = calcBagSales(order.products.bags);
                const percentageOfSales = orderOrigBagSales.value / savedVals.salesFromBags.value;
                const allocatedAmt =
                    USD(moneyFloorFromDouble(percentageOfSales * savedVals.allocationsForMulchBagSales));

                perUserReport[uid]['numBagsSold'] += order.products.bags;
                perUserReport[uid]['allocationFromBagsSold'] =
                    perUserReport[uid]['allocationFromBagsSold'].add(allocatedAmt);
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
                report.hasOwnProperty('numBagsSold')?report['numBagsSold'] : undefined,
                report.hasOwnProperty('numBagsSpreadSold')?report['numBagsSpreadSold'] : undefined,
                report.hasOwnProperty('deliveryMins')?report['deliveryMins'] : undefined,
                report.hasOwnProperty('donations')?report['donations'] : undefined,
                report.hasOwnProperty('allocationFromBagsSold')?report['allocationFromBagsSold'] : undefined,
                report.hasOwnProperty('allocationFromBagsSpread')?report['allocationFromBagsSpread'] : undefined,
                report.hasOwnProperty('allocationsFromDelivery')?report['allocationsFromDelivery'] : undefined,
                report['allocationTotal']
            ]);
        }

        perScoutReportDataFields.push([
            'scoutTotals',
            'Scout Total Allocations',
            savedVals.bagsSold,
            savedVals.bagsSpread,
            savedVals.deliveryMinutes,
            totalDonations, // I could pull all the below from global data but they wanted cross check
            allocationFromBagsSold,
            allocationFromBagsSpread,
            allocationsFromDelivery,
            allocationsTotal
        ]);

        const perScoutReport = [];
        for (const field of perScoutReportDataFields) {
            perScoutReport.push(
                <tr key={field[0]}>
                    <td>{field[1]}</td>
                    <td>{field[2] ? field[2] : ''}</td>
                    <td>{field[3] ? field[3] : ''}</td>
                    <td>{field[4] ? field[4] : ''}</td>
                    <td>{field[5] ? field[5].format() : ''}</td>
                    <td>{field[6] ? field[6].format() : ''}</td>
                    <td>{field[7] ? field[7].format() : ''}</td>
                    <td>{field[8] ? field[8].format() : ''}</td>
                    <td>{field[9] ? field[9].format() : ''}</td>
                </tr>
            );
        }

        setReportCard(
            <div className="card">
                <h5 className="card-header justify-content-center text-center">Allocation Report</h5>
                <div className="card-body">
                    <form onSubmit={onReleaseFundsFormSubmission}>
                        <div className="table-responsive" id="fundsReleaseTables">
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
                                </thead>
                                <tbody>
                                    {perScoutReport}
                                </tbody>
                            </table>
                        </div>
                        <button type="submit" className="btn btn-primary invisible my-2 float-end"
                                id="releaseFundsBtn"
                                data-bs-toggle="tooltip"
                                data-report-fields={perScoutReportDataFields}
                                title="Release Report to Scouts">
                            Generate Data
                        </button>
                    </form>
                    <button type="button" className="btn reports-view-setting-btn ms-3"
                            onClick={onDownloadReport} data-bs-toggle="tooltip"
                            title="Download Report">
                        <svg className="bi" fill="currentColor">
                            <use xlinkHref={exportImg}/>
                        </svg>
                    </button>
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
                    <div className="row justify-content-center">
                        <div className="card mb-3" style={{'maxWidth': '30rem'}}>
                            <h5 className="card-header justify-content-center text-center">Allocation Calculations</h5>
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
                                                    <td>Min Allocations to Troop</td>
                                                    <td>{troopPercentage}</td>
                                                </tr>
                                                <tr>
                                                    <td>Max Allocations to Scouts</td>
                                                    <td>{scoutPercentage}</td>
                                                </tr>
                                                <tr>
                                                    <td colSpan="4">
                                                        <table className="table table-striped caption-top mb-0">
                                                            <caption>Scout Allocations</caption>
                                                            <tbody>
                                                                <tr>
                                                                    <td>Allocations to Scouts for Mulch Bag Sales</td>
                                                                    <td>{scoutSellingPercentage}</td>
                                                                </tr>
                                                                <tr>
                                                                    <td>Scouts Avg Allocation per Bag</td>
                                                                    <td>{perBagAvgEarnings}</td>
                                                                </tr>
                                                                <tr>
                                                                    <td>Allocations to Scouts for Delivery</td>
                                                                    <td>{scoutDeliveryPercentage}</td>
                                                                </tr>
                                                                <tr>
                                                                    <td>Total Delivery Minutes</td>
                                                                    <td>{deliveryMinutes}</td>
                                                                </tr>
                                                                <tr>
                                                                    <td>Scout Earnings per Delivery Minute</td>
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

                                    <button type="submit" className="btn btn-primary invisible my-2 float-end"
                                            id="generateReportsBtn"
                                            data-bs-toggle="tooltip"
                                            title="Generate Data">
                                        Generate Data
                                    </button>
                                </form>
                                <button type="button" className="btn reports-view-setting-btn ms-3"
                                        onClick={onDownloadRawData} data-bs-toggle="tooltip"
                                        title="Download RawData">
                                    <svg className="bi" fill="currentColor">
                                        <use xlinkHref={exportImg}/>
                                    </svg>
                                </button>
                            </div>
                        </div> {/* End of card */}
                    </div>
                    <div className="row">
                        {reportCard}
                    </div>
                </div>
            </>
        )}
    </>);
}
