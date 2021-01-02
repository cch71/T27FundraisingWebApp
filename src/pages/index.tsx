import React, { useState, useEffect } from "react"
import { Router, Link } from '@reach/router'
//import NavBar from "../components/navbar"
import AddNewOrderWidget from "../components/add_new_order_widget"
import auth from "../js/auth"
import { navigate } from "gatsby"
import {orderDb, LeaderBoardSummaryInfo} from "../js/ordersdb"
import {FundraiserConfig, downloadFundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"
import awsConfig from "../config"
import currency from "currency.js"

const NewOrder = React.lazy(() => import('./order_step_1'));
const Reports = React.lazy(() => import('./orders'));
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

/* function dynamicColors(){
 *     return ['SaddleBrown', 'DarkOliveGreen', 'Blue', 'Purple', 'SlateGrey', 'Yellow', 'Salmon'];
 * };
 *  */

const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });


async function enableReady(frConfig: FundraiserConfig, setOrderSummary) {
    const readyViewElm = document.getElementById('readyView');
    if (readyViewElm) {
        readyViewElm.style.display = "block";
    }

    const notReadyViewElm = document.getElementById('notReadyView');
    if (notReadyViewElm) {
        notReadyViewElm.className = "d-none";
    }
    const summaryArr=[];
    orderDb.getOrderSummary().then((summary: LeaderBoardSummaryInfo)=>{
        if (!summary) { return; }

        const topSellers = [];
        for (const [ranking, seller, amt] of summary.topSellers()) {
            topSellers.push(
                <tr key={ranking}>
                    <td className="py-1">{ranking}</td>
                    <td className="py-1">{frConfig.getUserNameFromId(seller)}</td>
                    <td className="py-1">{USD(amt).format()}</td>
                </tr>
            );
        }
        //console.log("TopSeller ${JSON.stringify(topSellers)}")
        let statIndex=0;
        const summaryStats = [];
        const userSummary = summary.userSummary();
        summaryStats.push(
            <li key={++statIndex} className="list-group-item border-0 py-1">
                You have collected {USD(userSummary.amountSold).format()} in sales
            </li>
        );
        if ('mulch' === frConfig.kind()) {
            summaryStats.push(
                <li key={++statIndex} className="list-group-item border-0 py-1">
                    You have sold {userSummary.bags} bags of mulch
                </li>
            );
            summaryStats.push(
                <li key={++statIndex} className="list-group-item border-0 py-1">
                    You have sold {userSummary.spreading} spreading jobs
                </li>
            );
        }

        if (0.0 < userSummary.donation.value) {
            summaryStats.push(
                <li key={++statIndex} className="list-group-item border-0 py-1">
                    You have collected {USD(userSummary.donation).format()} in donations
                </li>
            );
        }

        summaryStats.push(
            <li key={++statIndex} className="list-group-item border-0 py-1">
                Troop has sold {USD(summary.troopAmountSold()).format()}
            </li>
        );

        //console.log("Summary ${JSON.stringify(summaryStats)}")

        setOrderSummary(
            <div className="col-xs-1 d-flex justify-content-center">
                <div className="card">
                    <div className="card-body">
                        <h5 className="card-title">{frConfig.description()} Fundraiser</h5>
                        <h6>Summary for: {frConfig.getUserNameFromId(auth.getCurrentUserId())}</h6>
                        <ul className="list-group list-group-flush">
                            {summaryStats}
                        </ul>

                        <h6 className="my-2">Top Sellers:</h6>
                        <table className="table table-sm table-borderless table-responsive" id="topSellersTable">
                            <tbody>
                                {topSellers}
                            </tbody>
                        </table>

                        <h6>Sales by Patrol:</h6>
                        <div id="patrolStandingsChart"/>
                    </div>
                    <small muted>*updates may take up to 15 minutes</small>
                </div>
            </div>
        );


        // Draw Charts
        const drawCharts=()=>{

            const options = { is3D: true };

            const patrolStandingsData = new google.visualization.DataTable();
            patrolStandingsData.addColumn('string', 'Patrol Sales');
            patrolStandingsData.addColumn('number', 'Amount Sold');

            for (const [patrol, amount] of summary.patrolRankings()) {
                patrolStandingsData.addRow([patrol, amount.value]);
            }

            const patrolStandingsChart = new google.visualization.PieChart(
                document.getElementById('patrolStandingsChart'));
            patrolStandingsChart.draw(patrolStandingsData, options);


        };
		// Load the Visualization API and the corechart package.
		google.charts.load('current', {'packages':['corechart']});
		// Set a callback to run when the Google Visualization API is loaded.
		google.charts.setOnLoadCallback(drawCharts);
    });
}

const Home = ()=>{

    const [orderSummary, setOrderSummary] = useState();
    useEffect(() => {
		const onLoadComponent = async ()=>{
			const [isValidSession, session] = await auth.getSession();
            if (!isValidSession) {
                // If no active user go to login screen
                navigate('/signon/');
                return;
            }
            console.log(`Active User: ${auth.getCurrentUserId()}`);

            const authToken = await auth.getAuthToken();

            try {
                const frConfig = getFundraiserConfig();
                await enableReady(frConfig, setOrderSummary);
            } catch(err: any) {
                const loadedConfig = await downloadFundraiserConfig(authToken);
                if (!loadedConfig) {
                    throw(new Err("Failed to load session fundraising config"));
                }
                await enableReady(loadedConfig, setOrderSummary);
            }

		};

		onLoadComponent()
			.then()
			.catch((err)=>{
				if ('Invalid Session'===err.message) {
					navigate('/signon/');
					return;
				} else {
					console.error(err);
				}
			});
    }, []);

    return (
        <div>
            <div id="notReadyView" className='col-xs-1 d-flex justify-content-center' >
                <div className="spinner-border" role="status">
                    <span className="visually-hidden">Loading...</span>
                </div>
            </div>
            <div id="readyView" style={{display: 'none'}}>
                {orderSummary}
                <AddNewOrderWidget/>
            </div>
        </div>
    );
}
//<NavBar/>


const IndexPage = ()=>{
    return(
        <Router>
            <Home path="/" />
            <LazyComponent Component={Reports} path="/orders/" />
            <LazyComponent Component={NewOrder} path="/order_step_1/" />
            <LazyComponent Component={SignOn} path="/signon/" />
        </Router>
    );
};



export default IndexPage;
