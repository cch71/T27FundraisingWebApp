import React, { useState, useEffect } from "react"
import { Router, Link } from '@reach/router'
import AddNewOrderWidget from "../components/add_new_order_widget"
import auth from "../js/auth"
import { navigate } from "gatsby"
import {orderDb, LeaderBoardSummaryInfo} from "../js/ordersdb"
import {FundraiserConfig, downloadFundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"
import awsConfig from "../config"
import currency from "currency.js"

const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });

async function showSummary(frConfig: FundraiserConfig, setOrderSummary) {
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
                You collected {USD(userSummary.amountSold).format()} in total
            </li>
        );
        if (0.0 < userSummary.donation.value) {
            summaryStats.push(
                <li key={++statIndex} className="list-group-item border-0 py-1">
                    You collected {USD(userSummary.donation).format()} in donations
                </li>
            );
        }
        if ('mulch' === frConfig.kind()) {
            summaryStats.push(
                <li key={++statIndex} className="list-group-item border-0 py-1">
                    You sold {userSummary.bags} bags of mulch
                </li>
            );
            summaryStats.push(
                <li key={++statIndex} className="list-group-item border-0 py-1">
                    You sold {userSummary.spreading} spreading jobs
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
            <div>
                <div className="justify-content-center text-center">
                    <h6>{frConfig.description()} Fundraiser</h6>
                    <div className="col-xs-1 d-flex justify-content-center">
                        <div className="row">

                            <div className="col-lg-4">
                                <div className="card" id="orderOwnerSummaryCard">
                                    <div className="card-header">
                                        Summary for: {frConfig.getUserNameFromId(auth.getCurrentUserId())}
                                    </div>
                                    <div className="card-body text-start">
                                        <small muted>*updates may take up to 15 minutes</small>
                                        <ul className="list-group list-group-flush sm-owner-summary"
                                            id="orderOwnerSummaryList">
                                            {summaryStats}
                                        </ul>
                                    </div>
                                </div>
                            </div>

                            <div className="col-lg-4">
                                <div className="card" id="topSellersCard">
                                    <div className="card-header">Top Sellers:</div>
                                    <div className="card-body text-start">
                                        <table className="table table-sm table-borderless table-responsive"
                                               id="topSellersTable">
                                            <tbody>
                                                {topSellers}
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                            </div>

                            <div className="col-lg-4">
                                <div className="card" id="patrolStandingsChartCard">
                                    <div className="card-header">Sales by Patrol:</div>
                                    <div className="card-body">
                                        <div id="patrolStandingsChart"/>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <AddNewOrderWidget/>
            </div>
        );


        // Draw Charts
        const drawCharts=()=>{

            const options = {
                is3D: true,
                legend: 'left'
            };

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


function showSignOn(setContent) {
    //If you got here then lets go ahean and sign out if already signed in.
    orderDb.setActiveOrder(); // Reset active order
    jQuery('#primaryNavBar').hide();


    const onFormSubmission = (event: any) => {
        const submitBtn = document.getElementById('loginBtn');
        const submitSpinner = document.getElementById('loginBtnSpinny');
        submitBtn.disabled=true;
        submitSpinner.style.display = "inline-block";

        const form = event.currentTarget;
        event.preventDefault();
        event.stopPropagation();

        const loginId = form[0].value;
        const pw = form[1].value;

        if (!loginId) {
            form[0].classList.add('is-invalid');
        } else {
            form[0].classList.remove('is-invalid');
        }

        if (!pw) {
            form[1].classList.add('is-invalid');
        } else {
            form[1].classList.remove('is-invalid');
        }

        if (!pw || !loginId) {
            form.classList.add('is-invalid');
            return;
        }

        //console.log(`Form submttted uid: ${loginId}`)

        const onSuccess=(autoInfo: any)=>{
            console.log(autoInfo);
            form.classList.remove('is-invalid');
            jQuery('#primaryNavBar').hide();
            window.location.reload(false);
        };
        const onFailure=()=>{
            console.error("authenticaiton Error");
            submitBtn.disabled=false;
            submitSpinner.style.display = "none";
            form.classList.add('is-invalid');
        };
        auth.signIn(loginId, pw, onSuccess, onFailure);
    };

    setContent(
        <div id="signOn" className="col-xs-1 d-flex justify-content-center">
            <div className="card my-5">
                <h4 className="card-header">
                    Troop 27 Fundraiser Sign On
                </h4>
                <div className="card-body">
                    <form className="needs-validation" noValidate onSubmit={onFormSubmission}>
                        <div className="row mb-3">
                            <div className="form-floating">
                                <input type="text" className="form-control" id="formLoginId"
                                       aria-describedby="loginIdHelp" placeholder="Enter Login ID"
                                       required />
                                <label htmlFor="formLoginId" className="ms-2">Enter Login ID</label>
                            </div>
                        </div>
                        <div className="row mb-3">
                            <div className="form-floating">
                                <input type="password" className="form-control"
                                       id="formPassword" placeholder="Password"
                                       required/>
                                <label htmlFor="formPassword" className="ms-2">Enter Password</label>
                            </div>
                        </div>
                        <div className="d-flex justify-content-end">
                            <button type="submit" id="loginBtn" className="btn btn-primary">Submit
                                <span className="spinner-border spinner-border-sm me-1" role="status"
                                      aria-hidden="true" id="loginBtnSpinny" style={{display: "none"}} />
                            </button>
                        </div>
                    </form>
                    <div className="invalid-feedback">
                        *Invalid Username or Password
                    </div>
                    <small id="loginIdHelp" className="form-text text-muted">
                        For questions contact the Fundraising Coordinator
                    </small>
                </div>
            </div>
        </div>
    );
};

const Home = ()=>{

    const [isLoading, setIsLoading] = useState(false);
    const [content, setContent] = useState();
    useEffect(() => {
        setIsLoading(true);
        const onAsyncView = async ()=>{
            const [isValidSession, session] = await auth.getSession();
            if (!isValidSession) {
                // If no active user go to login screen
                showSignOn(setContent);
            } else {
                console.log(`Active User: ${auth.getCurrentUserId()}`);

                const authToken = await auth.getAuthToken();

                try {
                    const frConfig = getFundraiserConfig();
                    await showSummary(frConfig, setContent);
                } catch(err: any) {
                    const loadedConfig = await downloadFundraiserConfig(authToken);
                    if (!loadedConfig) {
                        throw(new Err("Failed to load session fundraising config"));
                    }
                    await showSummary(loadedConfig, setContent);
                }
            }
            setIsLoading(false);
        };

        onAsyncView()
            .then()
            .catch((err)=>{
                if ('Invalid Session'===err.message) {
                    showSignOn(setContent);
                } else {
                    console.error(err);
                    alert(err);
                }
            });
    }, []);

    return (
        <div id="indexPage">
            {isLoading ? (
                <div id="notReadyView" className='col-xs-1 d-flex justify-content-center' >
                    <div className="spinner-border" role="status">
                        <span className="visually-hidden">Loading...</span>
                    </div>
                </div>
            ) : (
                <>{content}</>
            )}
        </div>
    );
}

export default Home;
