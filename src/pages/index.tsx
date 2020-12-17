import React, { useState, useEffect } from "react"
import { Router, Link } from '@reach/router';
import NavBar from "../components/navbar"
import auth from "../js/auth"
import { navigate } from "gatsby"
import {orderDb, SummaryInfo} from "../js/ordersdb"
import {FundraiserConfig, downloadFundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"
import awsConfig from "../config"
import currency from "currency.js"
import {GoogleCharts} from 'google-charts';

const NewOrder = React.lazy(() => import('./order_step_1'));
const SignOn = React.lazy(() => import('./signon'));

const LazyComponent = ({ Component, ...props }) => (
    <React.Suspense fallback={'<p>Loading...</p>'}>
        <Component {...props} />
    </React.Suspense>
);


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


const Home = ()=>{
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

            const enableReady = (frConfig: FundraiserConfig | null)=>{
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

                    const topSellers = [];
                    for (const [ranking, seller, amt] of summary.topSellers()) {
                        topSellers.push(
                            <tr key={ranking}>
                                <td className="py-1">{ranking}</td>
                                <td className="py-1">{seller}</td>
                                <td className="py-1">{amt}</td>
                            </tr>
                        );
                    }

                    let statIndex=0;
                    const summaryStats = [];
                    summaryStats.push(
                        <li key={++statIndex} className="list-group-item border-0 py-1">
                            You have {summary.totalNumOrders()} orders
                            and collected {USD(summary.totalAmountSold()).format()}
                        </li>
                    );

                    for (const stat of summary.frSpecificSummaryReport()) {
                        summaryStats.push(
                            <li key={++statIndex} className="list-group-item border-0 py-1">
                                {stat}
                            </li>
                        );
                    }
                    summaryStats.push(
                        <li key={++statIndex} className="list-group-item border-0 py-1">
                            Troop has sold {USD(summary.totalTroopSold()).format()}
                        </li>
                    );


                    setOrderSummary(
                        <div className="col-xs-1 d-flex justify-content-center">
                            <div className="card">
                                <div className="card-body">
                                    <h5 className="card-title">{frConfig.description()} Fundraiser</h5>
                                    <h6>Summary for: {summary.userName()}</h6>
                                    <ul className="list-group list-group-flush">{summaryStats}</ul>

                                    <h6 className="my-2">Top Sellers:</h6>
                                    <table className="table table-bordered"><tbody>{topSellers}</tbody></table>

                                    <h6>Sales by Patrol:</h6>
                                    <div id="patrolStandingsChart"/>
                                </div>
                            </div>
                            <button type="button"
                                    className="btn btn-outline-info add-order-btn"
                                    onClick={addNewOrder}>
                                +
                            </button>
                        </div>
                    );


                    // Draw Charts
                    const drawCharts=()=>{

                        const options = { is3D: true };

                        const patrolStandingsData = new GoogleCharts.api.visualization.DataTable();
                            patrolStandingsData.addColumn('string', 'Patrol Sales');
                        patrolStandingsData.addColumn('number', 'Amount Sold');

                        for (const [patrol, amount] of summary.patrolRankings()) {
                            patrolStandingsData.addRow([patrol, amount.value]);
                        }

                        const patrolStandingsChart = new GoogleCharts.api.visualization.PieChart(
                            document.getElementById('patrolStandingsChart'));
                        patrolStandingsChart.draw(patrolStandingsData, options);


                    };
                    GoogleCharts.load(drawCharts);
                });
            };


            try {
                const frConfig = getFundraiserConfig();
                enableReady(frConfig);
            } catch(err: any) {
                try {
                    downloadFundraiserConfig(authToken).then((loadedConfig: FundraiserConfig | null)=>{
                        if (null===loadedConfig) {
                            alert("Failed to load session fundraising config");
                        }
                        enableReady(loadedConfig);
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

    const onTestAuth = ()=>{
        auth.getAuthToken().then((authToken: any)=>{


            //https://t27fundraiser.s3.amazonaws.com/T27FundraiserConfig.json

            fetch('https://t27fundraiser.s3.amazonaws.com/T27FundraiserConfig.json', {
                method: 'get',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': authToken
                }
            }).then((resp: any)=>{
                if (!resp.ok) {
                    resp.text().then((bodyStr)=>{
                        throw Error(`HTTP Resp Error: ${resp.status} ${bodyStr}`);
                    });
                } else {
                    return resp.json()
                }
            }).then((jresp: any)=>{
                if (jresp) {
                    console.log(`FrConfig: ${JSON.stringify(jresp)}`);
                }
            }).catch((error)=>{
                console.error(error);
            });


            /* fetch(awsConfig.api.invokeUrl + '/users', {
             *     method: 'post',
             *     headers: {
             *         'Content-Type': 'application/json',
             *         'Authorization': authToken
             *     },
             *     body: JSON.stringify({'cmd': 'GET_USER', 'userId': 'scout1'})
             * }).then((resp: any)=>{
             *     if (!resp.ok) {
             *         resp.text().then((bodyStr)=>{
             *             throw Error(`HTTP Resp Error: ${resp.status} ${bodyStr}`);
             *         });
             *     } else {
             *         return resp.json()
             *     }
             * }).then((jresp: any)=>{
             *     if (jresp) {
             *         console.log(`GetUser: ${JSON.stringify(jresp)}`);
             *     }
             * }).catch((error)=>{
             *     console.error(error);
             * });
             */
        });
    };

    const onResetUsers = (event: any)=>{
        event.preventDefault();
        event.stopPropagation();

        const files = event.currentTarget.files;
        if (files.length) {
            const file = files[0];
            const reader = new FileReader();
            /* reader.onload = ((aImg)=>{
             *     return function(e) { aImg.src = e.target.result; }; })(img); */
            reader.addEventListener('load', (fileLoadEvent: any) => {
                const contents = JSON.parse(fileLoadEvent.target.result);
                console.log(`FileLoad  ${JSON.stringify(contents)}`);
            });
            reader.readAsBinaryString(file);

        }


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

                {/* <button type="button" className="btn btn-primary my-2" onClick={onTestAuth}>
                    Test Auth
					</button>

					<input type="file" id="resetUsers" onChange={onResetUsers} />
				  */}
                {orderSummary}
            </div>
        </div>
    );
}


const IndexPage = ()=>{
    return(
        <Router>
            <Home path="/" />
            <LazyComponent Component={NewOrder} path="/order_step_1/" />
            <LazyComponent Component={SignOn} path="/signon/" />
        </Router>
    );
};



export default IndexPage;
