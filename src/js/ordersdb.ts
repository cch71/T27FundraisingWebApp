import currency from "currency.js";
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config";
import awsConfig from "../config";
import auth from "../js/auth";
import { v4 as uuidv4 } from 'uuid';

/////////////////////////////////////////////
//
class OrderMetaFields{
    isLoadedFromDb?: boolean;
    isReadOnly?: boolean;
    origOrderOwner?: string;
}

/////////////////////////////////////////////
//
class Order {
    // Always should have some initialization
    readonly orderId: string;
    totalAmt: currency;

    isVerified?: boolean;
    doDeleteOrder?: boolean;
    orderOwner?: string;
    firstName?: string;
    lastName?: string;
    phone?: string;
    email?: string;
    addr1?: string;
    addr2?: string;
    specialInstructions?: string;
    neighborhood?: string;
    products?: Record<string, number>;
    donation?: currency;
    cashPaid?: currency;
    checkPaid?: currency;
    checkNums?: string;
    spreaders?: Array<string>;
    doCollectMoneyLater?: boolean;

    meta: OrderMetaFields|undefined = new OrderMetaFields();

    constructor(order?: any) {
        if (!order) {
            this.orderId = uuidv4();
            this.totalAmt = currency(0.0);
        } else {
            //console.log(`OrdersDB New order from: ${JSON.stringify(order)}`);

            Object.keys(order).forEach((key: string, _: number)=>{
                if ('cashPaid' === key ||
                    'checkPaid' === key ||
                    'donation' === key ||
                    'productsCost' === key ||
                    'totalAmt' === key)
                {
                    this[key] = currency(order[key]);
                } else {
                    this[key] = order[key];
                }
            });

            this.meta.origOrderOwner = order.orderOwner;
        }
    }

    serializeForSubmission(): string {
        // Make a copy to for submission since if it fails we may still need meta
        const submissionOrder = JSON.parse(JSON.stringify(this));
        if (!submissionOrder.orderOwner) {
            submissionOrder['orderOwner'] = auth.currentUser().getUsername();
        }
        delete submissionOrder['meta'];
		console.log(`Submitting Order: ${JSON.stringify(submissionOrder, null, '\t')}`);
        return JSON.stringify(submissionOrder);
    }
}

/////////////////////////////////////////////
//
interface OrderListItem<T> {
    orderId: string;
    firstName: string;
    lastName: string;
    addr1: string;
    addr2?: string;
    neighborhood: string;
    phone: string;
    email?: string;
    amountTotal: T;
}

/////////////////////////////////////////////
//
interface OrderSpradingComplete {
    orderId: string,
    orderOwner: string,
    spreaders: Array<string>
}

// const hashStr = (val: string): string {
//     let hash = 0, i, chr;
//     for (let i = 0; i < val.length; i++) {
//         chr   = val.charCodeAt(i);
//         hash  = ((hash << 5) - hash) + chr;
//         hash |= 0; // Convert to 32bit integer
//     }
//     return hash;
// }

/////////////////////////////////////////////
//
class LeaderBoardUserSummary {
    public amountSold: currency = currency(0);
    public donations: currency = currency(0);
    public spreading: number = 0;
    public bags: number = 0;

}

/////////////////////////////////////////////
//
class LeaderBoardSummaryInfo {
    public readonly areFundsReleased: boolean = false;
    private users_: Array<[string, currency]> = [];
    private troopSales_: currency = currency(0);
    private patrolInfo_: Record<string, currency> = {};
    private userInfo_: LeaderBoardUserSummary = new LeaderBoardUserSummary();

    /////////////////////////////////////////////
    //
    constructor(summaryResp: any, private userId_: string, frConfig: FundraiserConfig) {
        this.areFundsReleased = summaryResp.areFundsReleased;

        for (const [uid, summary] of Object.entries(summaryResp.perUserSummary)) {
            const totAmt = currency(summary.totalAmtCollected);
            this.troopSales_ = this.troopSales_.add(totAmt)
            this.users_.push([uid, totAmt]);
            const patrolName = frConfig.getPatrolNameFromId(uid);
            if (!this.patrolInfo_.hasOwnProperty(patrolName)) { 
                this.patrolInfo_[patrolName] = currency(0);
            }
            this.patrolInfo_[patrolName] = this.patrolInfo_[patrolName].add(totAmt);
            if (uid===userId_) {

                this.userInfo_.amountSold = totAmt;
                
                if (summary.hasOwnProperty('donations')) {
                    this.userInfo_.donations = currency(summary['donations']);
                }

                if (summary.hasOwnProperty('numBagsSold')) {
                    this.userInfo_['bags'] = summary['numBagsSold'];
                }

                if (summary.hasOwnProperty('numBagsSpreadSold')) {
                    this.userInfo_['spreading'] = currency(summary['numBagsSpreadSold']);
                }
            }
        }

        this.users_.sort((r,l)=>{ return(l[1].value - r[1].value); });
    }

    /////////////////////////////////////////////
    //
    userSummary(): LeaderBoardUserSummary {
        return this.userInfo_;
    }

    /////////////////////////////////////////////
    //
    *patrolRankings(): Generator<[string, currency]> {
        for (const [patrolName, amountSold] of Object.entries(this.patrolInfo_)) {
            yield [patrolName, amountSold];
        }
    }

    /////////////////////////////////////////////
    //
    *topSellers(): Generator<[number, string, string]> {
        //console.log(`Sum Resp: ${JSON.stringify(this.summaryResp_, null, '\t')}`);
		    const usersLen = (10 < this.users_.length) ? 10 : this.users_.length;
        for (let idx=0; idx < usersLen; ++idx) {
            yield [idx+1, this.users_[idx][0], this.users_[idx][1]];
        }
    }

    /////////////////////////////////////////////
    //
    userId(): string { return this.userId_; }

    /////////////////////////////////////////////
    //
    troopAmountSold(): currency {
        return this.troopSales_;
    }
}

/////////////////////////////////////////////
//
class OrderDb {
    private currentOrder_?: Order;
    private submitOrderPromise_?: Promise<void>;

    constructor() {}

    /////////////////////////////////////////
    //
    setActiveOrder(order?: Order, isReadOnly?: boolean) {
        this.currentOrder_ = order;
        if (order) {
            this.currentOrder_.meta.isLoadedFromDb = true;
            this.currentOrder_.meta.isReadOnly = isReadOnly;
        }
    }

    /////////////////////////////////////////
    //
    newActiveOrder(): Order {
        this.currentOrder_ = new Order();
        return this.currentOrder_;
    }

    /////////////////////////////////////////
    //
    getActiveOrder(): Order|undefined {
        return this.currentOrder_;
    }

    /////////////////////////////////////////
    //
    // Todo need to define summary type
    getOrderSummary(): Promise<LeaderBoardSummaryInfo>  {
        return new Promise(async (resolve, reject)=>{
            try {
                const userId = auth.currentUser().getUsername();
                const authToken = await auth.getAuthToken();
                const frConfig = await getFundraiserConfig();

                console.log(`OrderDB Query Parms: {}`);
                const resp = await fetch(awsConfig.api.invokeUrl + '/leaderboard', {
                    method: 'post',
                    headers: {
                        'Content-Type': 'application/json',
                        Authorization: authToken
                    }
                });

                if (!resp.ok) { // if HTTP-status is 200-299
                    const errRespBody = await resp.text();
                    throw new Error(`LeaderBoard Req error: ${resp.status}  ${errRespBody}`);
                } else {
                    const summaryInfo = await resp.json();
                    //console.log(`SummaryInfo: ${JSON.stringify(summaryInfo, null, '\t')}`)
                    resolve(new LeaderBoardSummaryInfo(summaryInfo, userId, frConfig));
                }

            } catch(error) {
                console.error(error);
                const leaderboardDefault = {
                    'patrols': {},
                    'troop': {},
                    'users': []
                };
                resolve(new LeaderBoardSummaryInfo(leaderboardDefault, userId));
            }
        });
    }

    /////////////////////////////////////////
    //
    getOrderFromId(orderId: string, orderOwner?: string): Promise<Order|undefined> {
        return new Promise((resolve)=>{
            let params = { orderId: orderId };
            if (orderOwner) { params['orderOwner'] = orderOwner}
            this.query(params).then((orders: Array<any>)=>{
                if (orders.length) {
                    const order = orders[0];
                    //console.log(`OrdersDB OrderFromId Found: ${JSON.stringify(order)}`);
                    resolve(new Order(order));
                } else {
                    resolve();
                }
            });
        });
    }

    /////////////////////////////////////////
    //
    deleteOrder(orderId: string, orderOwner: string): Promise<void> {
        return new Promise(async (resolve, reject)=>{
            try {
                const authToken = await auth.getAuthToken();
                const paramStr = JSON.stringify({
                    doDeleteOrder: true,
                    orderOwner: orderOwner,
                    orderId: orderId
                });

                //console.log(`OrderDB Query Parms: ${paramStr}`);
                const resp = await fetch(awsConfig.api.invokeUrl + '/upsertorder', {
                    method: 'post',
                    headers: {
                        'Content-Type': 'application/json',
                        Authorization: authToken
                    },
                    body: paramStr
                });

                if (!resp.ok) { // if HTTP-status is 200-299
                    const errRespBody = await resp.text();
                    const errStr = `Query error: ${resp.status}  ${errRespBody}`;
                    reject(new Error(errStr));
                } else {
                    resolve();
                }
            } catch(error) {
                console.error(error);
                const errStr = `Delete Order error: ${error.message}`;
                reject(error);
            }
        });
    }


    /////////////////////////////////////////
    //
    submitActiveOrder(): Promise<void> {
        if (!this.submitOrderPromise_) {
            this.submitOrderPromise_ = new Promise(async (resolve, reject)=>{
                if (!this.currentOrder_) {
                    this.submitOrderPromise_ = undefined;
                    reject(new Error("There is no active order"));
                    return;
                }

                const handleErr = (err: any)=>{
                    this.submitOrderPromise_ = undefined;
                    reject(err);
                };

                try {
                    const authToken = await auth.getAuthToken();
                    const origOrderOwner = this.currentOrder_.meta?.origOrderOwner;
                    const orderOwner = this.currentOrder_.orderOwner;
                    if (origOrderOwner && origOrderOwner!==orderOwner) {
                        await this.deleteOrder(this.currentOrder_.orderId, origOrderOwner);
                    }

                    const paramStr = this.currentOrder_.serializeForSubmission();
                    const resp = await fetch(awsConfig.api.invokeUrl + '/upsertorder', {
                        method: 'post',
                        headers: {
                            'Content-Type': 'application/json',
                            Authorization: authToken
                        },
                        body: paramStr
                    });

                    if (!resp.ok) { // if HTTP-status is 200-299
                        const errRespBody = await resp.text();
                        handleErr(
                            new Error(`Failed upserting order id: ${resp.status} reason: ${errRespBody}`));
                    } else {
                        this.submitOrderPromise_ = undefined;
                        // Order Submited so reset active order
                        this.setActiveOrder();
                        resolve();
                    }
                } catch(err) {
                    const errStr = `Failed req upserting order err: ${err.message}`;
                    handleErr(err);
                }
            });
        }

        return this.submitOrderPromise_;

    }


    /////////////////////////////////////////
    //
    submitSpreadingComplete(spreadingCompleteParams: OrderSpradingComplete): Promise<void> {
        return new Promise(async (resolve, reject)=>{
            const handleErr = (err: any)=>{
                reject(err);
            };

            try {
                const authToken = await auth.getAuthToken();

                const paramStr = JSON.stringify(spreadingCompleteParams);
                const resp = await fetch(awsConfig.api.invokeUrl + '/upsertorder', {
                    method: 'post',
                    headers: {
                        'Content-Type': 'application/json',
                        Authorization: authToken
                    },
                    body: paramStr
                });

                if (!resp.ok) { // if HTTP-status is 200-299
                    const errRespBody = await resp.text();
                    handleErr(
                        new Error(`Failed upserting order id: ${resp.status} reason: ${errRespBody}`));
                } else {
                    resolve();
                }
            } catch(err) {
                const errStr = `Failed req upserting order err: ${err.message}`;
                handleErr(err);
            }
        });
    }

    /////////////////////////////////////////
    //
    submitVerification(spreadingCompleteParams: OrderSpradingComplete): Promise<void> {
        return new Promise(async (resolve, reject)=>{
            const handleErr = (err: any)=>{
                reject(err);
            };

            try {
                const authToken = await auth.getAuthToken();

                const paramStr = JSON.stringify(spreadingCompleteParams);
                const resp = await fetch(awsConfig.api.invokeUrl + '/upsertorder', {
                    method: 'post',
                    headers: {
                        'Content-Type': 'application/json',
                        Authorization: authToken
                    },
                    body: paramStr
                });

                if (!resp.ok) { // if HTTP-status is 200-299
                    const errRespBody = await resp.text();
                    handleErr(
                        new Error(`Failed upserting order id: ${resp.status} reason: ${errRespBody}`));
                } else {
                    resolve();
                }
            } catch(err) {
                const errStr = `Failed req upserting order err: ${err.message}`;
                handleErr(err);
            }
        });
    }

    /////////////////////////////////////////
    //
    query(params: any|undefined): Promise<Array<any>> {
        return new Promise(async (resolve, reject)=>{
            try {
                if (!params) { params = {}; }
                const authToken = await auth.getAuthToken()

                if (!params.hasOwnProperty('orderOwner')) {
                    params['orderOwner'] = auth.currentUser().getUsername();
                }
                const paramStr = JSON.stringify(params);
                console.log(`OrderDB Query Parms: ${paramStr}`);
                const resp = await fetch(awsConfig.api.invokeUrl + '/queryorders', {
                    method: 'post',
                    headers: {
                        'Content-Type': 'application/json',
                        Authorization: authToken
                    },
                    body: paramStr
                });

                if (!resp.ok) { // if HTTP-status is 200-299
                    const errRespBody = await resp.text();
                    const errStr = `Query error: ${resp.status}  ${errRespBody}`;
                    reject(new Error(errStr));
                } else {
                    const ordersReturned: any = await resp.json();
                    //console.log(`OrdersDB Query Resp: ${JSON.stringify(ordersReturned)}`);
                    resolve(ordersReturned);
                }
            } catch(error) {
                console.error(error);
                const errStr = `Query req error: ${error.message}`;
                reject(error);
            }
        });
    }
}

const orderDb = new OrderDb()

export {orderDb, Order, OrderListItem, LeaderBoardSummaryInfo};
