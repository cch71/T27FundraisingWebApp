import currency from "currency.js"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"
import awsConfig from "../config"
import auth from "../js/auth"
import { v4 as uuidv4 } from 'uuid';


/////////////////////////////////////////////
//
class Order {
    // Always should have some initialization
    readonly orderId: string;
    totalAmt: currency;

    isReadOnly?: boolean;
    isValidated?: boolean;
    doDeleteOrder?: boolean;
    orderOwner?: string;
    firstName?: string;
    lastName?: string;
    phone?: string;
    email?: string;
    addr1?: string;
    addr2?: string;
    city?: string;
    state?: string;
    zip?: string;
    specialInstructions?: string;
    neighborhood?: string;
    products?: Record<string, number>;
    donation?: currency;
    cashPaid?: currency;
    checkPaid?: currency;
    checkNums?: string;
    doCollectMoneyLater?: boolean;

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
        }
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
interface LeaderBoardUserSummary {
    amountSold: number;
    orderOwner: string;
    donation?: number;
    spreading?: number;
    bags?: number;
}

/////////////////////////////////////////////
//
class LeaderBoardSummaryInfo {
    constructor(private summaryResp_: any, private userId_: string)
    {}

    userId(): string { return this.userId_; }
    troopAmountSold(): currency {
        return currency(this.summaryResp_.troop?.amountSold);
    }
    userSummary(): LeaderBoardUserSummary {
        for (const user_summary of this.summaryResp_.users) {
            if (this.userId() === user_summary.orderOwner) {
                user_summary.amountSold = currency(user_summary.amountSold);
                if (user_summary.donation) {
                    user_summary.donation = currency(user_summary.donation);
                }
                return user_summary;
            }
        }
        const defaultVal = {
            amountSold: currency(0),
            orderOwner: "",
            donation: currency(0),
        };

        defaultVal['bags'] = 0;
        defaultVal['spreading'] = 0;
        return defaultVal;
    }
    *topSellers(): Generator<[number, string, string]> {
        const users = this.summaryResp_.users;
        //console.log(`Sum Resp: ${JSON.stringify(this.summaryResp_, null, '\t')}`);
        for (let idx=0; idx < users.length; ++idx) {
            yield [idx+1, users[idx].orderOwner, currency(users[idx].amountSold)]
        }
    }

    *patrolRankings(): Generator<[string, currency]> {
        for (const patrol of Object.getOwnPropertyNames(this.summaryResp_.patrols)) {
            yield [patrol, currency(this.summaryResp_.patrols[patrol].amountSold)];
        }
    }
}

/////////////////////////////////////////////
//
class OrderDb {
    private readonly fundraiserConfig_: any;
    private currentOrder_?: Order;
    private submitOrderPromise_?: Promise<void>;

    constructor() {}

    /////////////////////////////////////////
    //
    setActiveOrder(order?: Order, isReadOnly?: boolean) {
        this.currentOrder_ = order;
        if (order) {
            this.currentOrder_.isReadOnly = isReadOnly;
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

                //console.log(`OrderDB Query Parms: ${paramStr}`);
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
                    resolve(new LeaderBoardSummaryInfo(summaryInfo, userId));
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
    getOrderFromId(orderId: string): Promise<Order|undefined> {
        return new Promise((resolve)=>{
            this.query({orderId: orderId}).then((orders: Array<any>)=>{
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
    deleteOrder(orderId: string): Promise<void> {
        return new Promise(async (resolve, reject)=>{
            try {
                auth.getAuthToken().then(async (authToken: string)=>{
                    const orderOwner = auth.currentUser().getUsername();
                    if (!orderOwner || !orderId) {
                        reject(new Error("Order ID or Login ID is invalid"));
                        return;
                    }
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
                }).catch((err: any)=>{
                    reject(err);
                });


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
                    auth.getAuthToken().then(async (authToken: string)=>{

                        this.currentOrder_['orderOwner'] = auth.currentUser().getUsername();
                        const paramStr = JSON.stringify(this.currentOrder_);
                        //console.log(`Updating Order: ${paramStr}`);
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
                            handleErr(new Error(`Failed upserting order id: ${resp.status} reason: ${errRespBody}`));
                        } else {
                            this.submitOrderPromise_ = undefined;
                            // Order Submited so reset active order
                            this.setActiveOrder();
                            resolve();
                        }
                    }).catch((err: any)=>{
                        handleErr(err);
                    });


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
    query(params: any|undefined): Promise<Array<any>> {
        return new Promise(async (resolve, reject)=>{
            try {
                if (!params) { params = {}; }
                const authToken = await auth.getAuthToken()

                if (!params.hasOwnProperty('orderOwner')) {
                    params['orderOwner'] = auth.currentUser().getUsername();
                }
                const paramStr = JSON.stringify(params);
                //console.log(`OrderDB Query Parms: ${paramStr}`);
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
