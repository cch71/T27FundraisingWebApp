import React, { useState, useEffect } from "react";
import AddNewOrderWidget from "../components/add_new_order_widget"
import {reportViews, genDeleteDlg, genSpreadingDlg, genConfirmDlg} from "../components/report_view"
import { navigate } from "gatsby";
import {orderDb, OrderListItem} from "../js/ordersdb";
import currency from "currency.js";
import auth from "../js/auth"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config";
import bootstrapIconSprite from "bootstrap-icons/bootstrap-icons.svg";

const exportImg = bootstrapIconSprite + "#cloud-download";
const reportSettingsImg = bootstrapIconSprite + "#gear";

const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });
const rprtStngDlgRt = 'reportViewSettingsDlg';
let reportSettingsDlg = undefined;
let lastAuthenticatedUser = undefined;

////////////////////////////////////////////////////////////////////
//
const showTheSelectedView = async (frConfig: FundraiserConfig) => {
    const genOption = (label, val)=>{
        const option = document.createElement("option");
        option.text = label;
        option.value = val ? val : label;
        return option;
    };

    const userSelElm = document.getElementById(`${rprtStngDlgRt}UserSelection`);
    const viewSelElm = document.getElementById(`${rprtStngDlgRt}ViewSelection`);

    const showView_ = ()=>{
        //Update the selected view label
        const selectedUser = userSelElm.options[userSelElm.selectedIndex].text;
        const selectedViewOpt = viewSelElm.options[viewSelElm.selectedIndex];
        const rvLabel = document.getElementById("reportViewLabel");
        const rvOrderOwner = document.getElementById("reportViewOrderOwner");
        console.log(`${selectedViewOpt.text}(${selectedUser})`);
        rvLabel.innerText = `${selectedViewOpt.text}`;
        rvOrderOwner.innerText = `${selectedUser}`;

        const params = {
            userId: userSelElm.options[userSelElm.selectedIndex].value
        };
        reportViews.showView(selectedViewOpt.value, frConfig, params);
    };

    // Check to see if Report Views User view has been initialized
    if (0===userSelElm.options.length) {
        const [views, users] = await reportViews.getViewOptions();
        for (const userInfo of users) {
            userSelElm.add(genOption(userInfo[1], userInfo[0]));
        }
        userSelElm.value = reportViews.getCurrentUserId();

        for (const reportView of views) {
            viewSelElm.add(genOption(reportView[0], reportView[1]));
        }
        viewSelElm.value = reportViews.getCurrentView();

        if (await auth.isCurrentUserAdmin()) {
            document.getElementById(`${rprtStngDlgRt}UserSelectionCol`).style.display = "inline-block";
        } else {
            document.getElementById(`${rprtStngDlgRt}UserSelectionCol`).style.display = "none";
        }
        showView_();
    } else {
        showView_();
    }

};

////////////////////////////////////////////////////////////////////
//
const genReportSettingsDlg = ()=>{
    return(
        <div className="modal fade" id={rprtStngDlgRt} tabIndex="-1" aria-labelledby={rprtStngDlgRt + "Title"} aria-hidden="true">
            <div className="modal-dialog modal-dialog-centered">
                <div className="modal-content">
                    <div className="modal-header">
                        <h5 className="modal-title" id={rprtStngDlgRt + "LongTitle"}>
                            Switch report view settings
                        </h5>
                        <button type="button" className="close" data-bs-dismiss="modal" aria-label="Close"></button>
                    </div>
                    <div className="modal-body">
                        <div className="container-sm">
                            <div className="row">
                                <div className="col-sm">
                                    <div className="form-floating">
                                        <select className="form-select" id={rprtStngDlgRt+"ViewSelection"}/>
                                        <label htmlFor={rprtStngDlgRt+"ViewSelection"}>
                                            Select Report View
                                        </label>
                                    </div>
                                </div>
                                <div className="col-sm" id={rprtStngDlgRt+"UserSelectionCol"}>
                                    <div className="form-floating">
                                        <select className="form-select" id={rprtStngDlgRt+"UserSelection"} />
                                        <label htmlFor={rprtStngDlgRt+"UserSelection"}>
                                            Select User
                                        </label>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div className="modal-footer">
                        <button type="button" className="btn btn-secondary"
                                data-bs-dismiss="modal">Cancel</button>
                        <button type="button" className="btn btn-primary"
                                data-bs-dismiss="modal" id={rprtStngDlgRt + "OnSave"}>
                            Save
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
};


////////////////////////////////////////////////////////////////////
//
const genCardBody = (frConfig: FundraiserConfig) => {
    const fullName = frConfig.getUserNameFromId(auth.getCurrentUserId());

    const onVewSettingsClick = ()=>{
        const dlgElm = document.getElementById(rprtStngDlgRt);
        reportSettingsDlg = new bootstrap.Modal(dlgElm, {
            backdrop: true,
            keyboard: true,
            focus: true
        });

        document.getElementById(rprtStngDlgRt+"OnSave").onclick = (event)=>{
            showTheSelectedView(frConfig);
        }

        reportSettingsDlg.show();
    };


    const onDownloadReportClick = ()=>{

        const csvData = reportViews.genCsvFromCurrent();

        const hiddenElement = document.createElement('a');
		    const blob = new Blob([csvData], { type: 'text/plain;charset=utf-8' });
		    hiddenElement.href = URL.createObjectURL(blob);
        hiddenElement.target = '_blank';
        hiddenElement.download = 'FundraisingReport.csv';
        hiddenElement.click();
    };


    return(
        <div className="card-body" id="cardReportBody">
            <h6 className="card-title ps-2" id="orderCardTitle">
                <ul className="list-group list-group-horizontal-sm">
                    <li className="list-group-item me-3">
                        <label className="text-muted pe-2">Report View:</label>
                        <div className="d-inline" id="reportViewLabel">Default</div>
                    </li>
                    <li className="list-group-item" id="orderOwnerLabel">
                        <label className="text-muted pe-2">Showing Orders for:</label>
                        <div className="d-inline" id="reportViewOrderOwner">{fullName}</div>
                    </li>
                </ul>
                <div id="reportViewSettings" className="float-end">
                    <button type="button" className="btn reports-view-setting-btn" onClick={onDownloadReportClick}
                            data-bs-toggle="tooltip" title="Download Current Report">
                        <svg className="bi" fill="currentColor">
                            <use xlinkHref={exportImg}/>
                        </svg>
                    </button>
                    <button type="button" className="btn reports-view-setting-btn" onClick={onVewSettingsClick}
                            data-bs-toggle="tooltip" data-bs-placement="left" title="Change Report View">
                        <svg className="bi" fill="currentColor">
                            <use xlinkHref={reportSettingsImg}/>
                        </svg>
                    </button>
                </div>
            </h6>

            <div id="reportsDataTable">
                <table  className="display responsive nowrap collapsed" role="grid" style={{width:"100%"}}/>
            </div>
            <div id="reportsRawTable">
            </div>

            <div className="spinner-border" role="status" id="orderLoadingSpinner">
                <span className="visually-hidden">Loading...</span>
            </div>
        </div>
    );
};

////////////////////////////////////////////////////////////////////
//
export default function orders(params: any) {

    const addNewOrder=()=>{
        console.log("Add new order");
        orderDb.newActiveOrder();
        navigate('/order_step_1/');
    };

    // Client-side Runtime Data Fetching
    const [newOrderButton, setNewOrderButton] = useState();
    const [cardBody, setCardBody] = useState();
    const [deleteDlg, setDeleteDlg] = useState();
    const [confirmDlg, setConfirmDlg] = useState();
    const [spreadDlg, setSpreadDlg] = useState();
    const [settingsDlg, setReportSettingsDlg] = useState();
    useEffect(() => {
        const onLoadComponent = async ()=>{
            const [isValidSession, session] = await auth.getSession();
            if (!isValidSession) {
                // If no active user go to login screen
                navigate('/');
                return;
            }

            if (lastAuthenticatedUser !== auth.getCurrentUserId()) {
                // We had a user login change
                console.log("Detected differentUser");
                reportViews.resetToDefault();
                lastAuthenticatedUser = auth.getCurrentUserId();
            }

            const frConfig = getFundraiserConfig();

            console.log("loaded FrConfig");

            console.log("Loading Gen Card Body");
            setCardBody(genCardBody(frConfig));
            setDeleteDlg(genDeleteDlg());
            setConfirmDlg(genConfirmDlg());
            setSpreadDlg(genSpreadingDlg(frConfig));
            setReportSettingsDlg(genReportSettingsDlg());


            await showTheSelectedView(frConfig);
            const [_, userGroups] = await auth.getUserIdAndGroups();
            const isAdmin = await auth.isCurrentUserAdmin();
            if (isAdmin || frConfig.isAddOrEditOrdersAllowed()) {
                setNewOrderButton(<AddNewOrderWidget/>);
            }

            if (isAdmin) {
				        document.getElementById("orderOwnerLabel").style.display = "none";
			      }

            // Enable all tooltips TODO: Disabled due to instability
            /* const tooltipTriggerList = [].slice.call(document.querySelectorAll('[data-bs-toggle="tooltip"]'))
             * tooltipList = tooltipTriggerList.map(function (tooltipTriggerEl) {
             *     return new bootstrap.Tooltip(tooltipTriggerEl, {
             *         boundary: 'window'
             *     });
             * }) */
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
        <div>
            { newOrderButton }
            <div className="col-xs-1 d-flex justify-content-center">
                <div className="card" style={{width: "100%"}}>
                    {cardBody}
                </div>
            </div>


            {deleteDlg}
            {settingsDlg}
            {spreadDlg}
            {confirmDlg}
        </div>
    );
}
