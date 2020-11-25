import React, { useState, useEffect } from "react"
import NavBar from "../components/navbar"
import auth from "../js/auth"
import { navigate } from "gatsby"
import {orderDb, SummaryInfo} from "../js/ordersdb"
import {FundraiserConfig, downloadFundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"
import awsConfig from "../config"
import currency from "currency.js"
import Chart from 'chart.js'


/* function *dynamicColors(): Generator<string> {
 *      var r = Math.floor(Math.random() * 255);
 *      * var g = Math.floor(Math.random() * 255);
 *      * var b = Math.floor(Math.random() * 255);
 *      * return "rgb(" + r + "," + g + "," + b + ")"; 
*     for (const c of ['red', 'green', 'blue', 'purple', 'yellow', 'brown', 'orange']) {
    *         yield c
    *     }
* };
*  */

function dynamicColors(){
    return ['SaddleBrown', 'DarkOliveGreen', 'Blue', 'Purple', 'SlateGrey', 'Yellow', 'Salmon'];
};


export default function home() {
    const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });

    const [orderSummary, setOrderSummary] = useState();
    useEffect(() => {
        auth.getSession().then((results)=>{
            const [isValidSession, session] = results;
            if (!isValidSession) {
                // If no active user go to login screen
                navigate('/signon/');
                return;
            }
            console.log(`Active User: ${auth.currentUserEmail()}`);

            const authToken = session.getIdToken().getJwtToken();
            
            const enableReady = ()=>{
                const readyViewElm = document.getElementById('readyView');
                if (readyViewElm) {
                    readyViewElm.style.display = "block";
                }

                const notReadyViewElm = document.getElementById('notReadyView');
                if (notReadyViewElm) {
                    notReadyViewElm.className = "d-none";
                }
                const summaryArr=[];
                orderDb.getOrderSummary().then((summary: SummaryInfo)=>{
                    if (!summary) { return; }
                    setOrderSummary(
                        <div className="col-xs-1 d-flex justify-content-center">
                            <div className="card">
                                <div className="card-body">
                                    <h5 className="card-title">Summary for: {summary.userName()}</h5>
                                    <div>You have {summary.totalNumOrders()} orders 
                                    and collected {USD(summary.totalAmountSold()).format()}</div>
                                    <div>Troop has sold {USD(summary.totalTroopSold()).format()}</div>
                                    <canvas id="yourPercentageChart"></canvas>
                                    <canvas id="patrolStandingsChart"></canvas>
                                </div>
                            </div>
                            <button type="button"
                                    className="btn btn-outline-light add-order-btn"
                                    onClick={addNewOrder}>
                                +
                            </button>
                        </div>
                    );

                    //Chart.defaults.global.legend.labels.boxWidth = '10';

                    // Chart Options
                    const chartOptions = {
                        responsive: true,
                        legend: {
                            position: 'left',
                        }
                    };
                    
                    // Your Pie Chart
                    const myStandingChart = new Chart('yourPercentageChart', {
                        type: 'pie',
                        data: {
                            labels: ['You', 'Troop'],
                            datasets: [{
                                data:[
                                    summary.totalAmountSold(),
                                    summary.totalTroopSold().subtract(summary.totalAmountSold())
                                ],
                                backgroundColor: ['Blue', 'DarkOliveGreen']
                            }]
                        },
                        options: chartOptions
                    });

                    // Patrol Pie Chart
                    const patrolData = {
                        labels: [],
                        datasets: [{
                            data:[],
                            backgroundColor: dynamicColors
                        }]
                    };
                    for (const [patrol, amount] of summary.patrolRankings()) {
                        patrolData.labels.push(patrol);
                        patrolData.datasets[0].data.push(amount);
                    }
                    const patrolPieChart = new Chart('patrolStandingsChart', {
                        type: 'pie',
                        data: patrolData,
                        options: chartOptions
                    });
                });
            };


            try {
                getFundraiserConfig();
                enableReady();                
            } catch(err: any) {
                try {
                    downloadFundraiserConfig(authToken).then((loadedConfig: FundraiserConfig | null)=>{
                        if (null===loadedConfig) {
                            alert("Failed to load session fundraising config");
                        }
                        enableReady();
                    });
                } catch(err: any) {
                    alert("Failed: " + err);
                }
            }
        });
    }, []);


    const addNewOrder = ()=>{
        console.log("Add new order");
        orderDb.newActiveOrder();
        navigate('/order_step_1/');
    };
    

    return (
        <div>
            <div id="notReadyView" className='col-xs-1 d-flex justify-content-center' >
                <div className="spinner-border" role="status">
                    <span className="sr-only">Loading...</span>
                </div>
            </div>
            <div id="readyView" style={{display: 'none'}}>
                <NavBar/>
                {orderSummary}
            </div>
        </div>
    );
}
