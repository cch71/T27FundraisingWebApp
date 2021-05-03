import React, { useState, useEffect } from "react";
import { navigate } from "gatsby";
import awsConfig from "../config";
import auth from "../js/auth";
import {orderDb, Order, OrdersForDeliveryDate} from "../js/ordersdb";
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config";
import currency from "currency.js";
import {onCurrencyFieldKeyPress, moneyFloor} from "../js/utils";
import CurrencyWidget from "../components/currency_widget";


import bootstrapIconSprite from "bootstrap-icons/bootstrap-icons.svg";

//const exportImg = bootstrapIconSprite + "#cloud-download";

const pad = (val)=>{return (val<10) ? '0' + val : val };
const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });

////////////////////////////////////////////////////
//
const getSpreadingPrice = (frConfig: FundraiserConfig): currency => {
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
	//console.log(`Saved Time Cards:\n${JSON.stringify(savedTimeCards, null, '\t')}`);
	let totalMins = 0;
	const timeCards = await getSavedTimeCards();
	for (const timeCard of timeCards) {
		if (timeCard.timeTotal) {
			let times = timeCard.timeTotal.split(":");
			totalMins += parseInt(times[1]) + (60 * parseInt(times[0]));
		}
	}
    return totalMins;
};

const savedVals = {
	spreadingTotal: USD(0),
	bagsSold: 0,
	deliveryMinutes: 0,
	bankDeposited: USD("$54,979.90"),
	mulchCost: USD("$22,319.70"),
}

////////////////////////////////////////////////////
//
export default function fundsRelease() {

    const [isLoading, setIsLoading] = useState(false);
    const [bankDeposited, setBankDeposited] = useState();
    const [mulchCost, setMulchCost] = useState();
    const [scoutTotalCollected, setScoutTotalCollected] = useState();
    const [spreadingTotal, setSpreadingTotal] = useState();
    const [mulchSalesGross, setMulchSalesGross] = useState();

    const [troopPercentage, setTroopPercentage] = useState();
    const [scoutPercentage, setScoutPercentage] = useState();
    const [scoutDeliveryPercentage, setScoutDeliveryPercentage] = useState();
    const [scoutSellingPercentage, setScoutSellingPercentage] = useState();

    const [bagsSold, setBagsSold] = useState();
    const [perBagsEarnings, setPerBagEarnings] = useState();
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

            let frConfig;
            try {
                frConfig = getFundraiserConfig();
            } catch(err) {
                console.error(`Failed loading fundraiser config going to main page`);
                navigate('/');
                return;
            }

            const pricePerBagToSpread = getSpreadingPrice(frConfig);

			savedVals.deliveryMinutes = await getDeliveryMinutes();

            const fieldNames = ["products.spreading", "products.bags", 'totalAmt']
            const orders = await orderDb.query({fields: fieldNames, orderOwner: 'any'});
			//console.log(`Orders:\n${JSON.stringify(orders, null, '\t')}`);
            let scoutCollected = currency(0.0);
            let bagsSpread = 0;
            for (const order of orders) {
                if (order.totalAmt) {
                    scoutCollected = scoutCollected.add(currency(order.totalAmt));
                }

                if (order.products?.spreading) {
                    bagsSpread += parseInt(order.products.spreading);
                }

                if (order.products?.bags) {
                    savedVals.bagsSold += parseInt(order.products.bags);
                }
            }

			savedVals.spreadingTotal = USD(pricePerBagToSpread.multiply(bagsSpread));
			setBankDeposited(savedVals.bankDeposited.format());
			setMulchCost(savedVals.mulchCost.format());
            setScoutTotalCollected(USD(scoutCollected).format());
            setSpreadingTotal(savedVals.spreadingTotal.format());
            setBagsSold(savedVals.bagsSold);
            setDeliveryMinutes(savedVals.deliveryMinutes);

        };

        onLoadComponent()
            .then(()=>{
                setIsLoading(false);
				onInputChange();
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
    const onInputChange = (event: any)=>{
        const bankDeposited = document.getElementById('formBankDeposited').value;
        const mulchCost = document.getElementById('formMulchCost').value;
		//TODO Save savedVals bankDeposited and mulchCost
        console.log(`BD: ${bankDeposited}, MS: ${mulchCost}, SP: ${savedVals.spreadingTotal}`);
        const totalGross = USD(bankDeposited).subtract(USD(savedVals.spreadingTotal)).subtract(USD(mulchCost));
        setMulchSalesGross(totalGross.format());

        setTroopPercentage(totalGross.multiply(.20).format());

        const scoutPercentage = totalGross.multiply(.80);
        setScoutPercentage(scoutPercentage.format());
        const [sellingPercentage, deliveryPercentage] = scoutPercentage.distribute(2);
        setScoutSellingPercentage(sellingPercentage.format());
        setScoutDeliveryPercentage(deliveryPercentage.format());

        setPerBagEarnings(USD(sellingPercentage/savedVals.bagsSold).format());
        setDeliveryEarnings(USD(deliveryPercentage/savedVals.deliveryMinutes).format());

        //console.log(`Scout Percentage: ${scoutPercentage}`);

        /* const [amt, isChanged] = moneyFloor(origAmt);
         * if (isChanged) {
         *     (document.getElementById('formDonationAmount') as HTMLInputElement).value = amt.toString();
         * }
         *
         * if (event.currentTarget.value) {
         *     (document.getElementById('formDonationSubmit') as HTMLButtonElement).disabled = false;
         * } else {
         *     (document.getElementById('formDonationSubmit') as HTMLButtonElement).disabled = true;
         * } */
    };

    ////////////////////////////////////////////////////
    //
    const onFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();

        /* const amountDue = currency((document.getElementById('formDonationAmount') as HTMLInputElement).value);

         * if (amountDue) {
         *     currentOrder['donation'] = amountDue;
         * } else {
         *     delete currentOrder['donation'];
         * }

         * navigate('/order_step_1/'); */
    }


    ////////////////////////////////////////////////////
    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <div className="card">
                <div className="card-body">
                    <h5 className="card-title justify-content-center text-center">Funds Release Page</h5>
                    {isLoading ? (
                        <div id="notReadyView" className='col-xs-1 d-flex justify-content-center' >
                            <div className="spinner-border" role="status">
                                <span className="visually-hidden">Loading...</span>
                            </div>
                        </div>
                    ) : (
                        <form onSubmit={onFormSubmission}>
                            <div className="row mb-2">
                                <div>Scout Total Collected: {scoutTotalCollected}</div>
                            </div>
                            <div className="row mb-2">
                                <CurrencyWidget id="formBankDeposited"
                                                defaultValue={bankDeposited}
                                                label="Amount Deposited in Bank"
                                                onInput={onInputChange}
                                />
                            </div>
                            <div className="row mb-2">
                                <CurrencyWidget id="formMulchCost"
                                                defaultValue={mulchCost}
                                                label="Amount Paid for Mulch"
                                                onInput={onInputChange}
                                />
                            </div>
                            <div className="row mb-2">
                                <div>Amount for Spreading: {spreadingTotal}</div>
                            </div>
                            <div className="row mb-2">
                                <div>Gross Mulch Sale Profits: {mulchSalesGross}</div>
                            </div>
                            <div className="row mb-2">
                                <div>Troop Percentage: {troopPercentage}</div>
                            </div>
                            <div className="row mb-2">
                                <div>Scout Percentage: {scoutPercentage}</div>
                                <div>Scout Delivery Percentage: {scoutDeliveryPercentage}</div>
                                <div>Scout Selling Percentage: {scoutSellingPercentage}</div>
                            </div>
                            <div className="row mb-2">
                                <div>Total Bags Sold: {bagsSold}</div>
                                <div>Scout Earnings per Bag Sold: {perBagsEarnings}</div>
                            </div>
                            <div className="row mb-2">
                                <div>Total Delivery Minutes: {deliveryMinutes}</div>
                                <div>Scout Earnings per Delivery Minute: {deliveryEarnings}</div>
                            </div>

                        </form>
                    )}
                </div>
            </div>
        </div>
    );
}
