import currency from "currency.js"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"
import awsConfig from "../config"
import auth from "../js/auth"
import { v4 as uuidv4 } from 'uuid';

/////////////////////////////////////////////
//
interface OrdersForDeliveryDate {
    amountDue: currency;
    kind: string;
    specialInstructions?: string;
    deliveryDateId?: string;
    items?: Map<string, number> //productId, numOrders
}

/////////////////////////////////////////////
//
class Order {
    // Always should have some initialization
    readonly orderId: string;
    cashPaid: currency;
    checkPaid: currency;
    amountTotal: currency;
    orderByDelivery: Map<string, OrdersForDeliveryDate>;


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
    neighborhood?: string;
    checkNumbers?: string;

    constructor(order?: any) {
        if (!order) {
            this.orderId = uuidv4();
            this.cashPaid = currency(0.0);
            this.checkPaid = currency(0.0);
            this.amountTotal = currency(0.0);
            this.orderByDelivery = new Map<string, OrdersForDeliveryDate>();
        } else {
            //console.log(`OrdersDB New order from: ${JSON.stringify(order)}`);

            Object.keys(order).forEach((key: string, _: number)=>{
                if ('cashPaid' === key ||
                    'checkPaid' === key ||
                    'amountTotal' === key)
                {
                    this[key] = currency(order[key]);
                } else if ('orderByDelivery' === key) {
                    this[key] = new Map<string, OrdersForDeliveryDate>();
                    Object.keys(order[key]).forEach((mapKey: string)=>{
                        const deliveryOrder: any = {};
                        //console.log(`!!!! ${mapKey}: ${JSON.stringify(order[key][mapKey])}`);
                        Object.keys(order[key][mapKey]).forEach((orderKey: string)=>{
                            const deliveryVal = order[key][mapKey][orderKey];
                            //console.log(`!!!!!!!! ${orderKey}: ${JSON.stringify(deliveryVal)}`);
                            if ('amountDue' === orderKey) {
                                deliveryOrder[orderKey] = currency(deliveryVal);
                            } else if ('items' === orderKey) {
                                deliveryOrder[orderKey] = new Map<string, number>(Object.entries(deliveryVal));
                            } else {
                                deliveryOrder[orderKey] = deliveryVal;
                            }
                        });
                        this[key].set(mapKey, deliveryOrder);
                    });
                } else {
                    this[key] = order[key];
                }
            });
            if (!order.hasOwnProperty('cashPaid')) {
                this['cashPaid'] = currency(0.0);
            }
            if (!order.hasOwnProperty('checkPaid')) {
                this['checkPaid'] = currency(0.0);
            }
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

const mockSummaryResults = {
    "totalTroopAmountSold": 25000.75,
    "patrolRanking": [
        {
            "patrol": "Bear",
            "amountSold": 19000.75,
        },{
            "patrol": "Black Dragon",
            "amountSold": 4000.00
        },{
            "patrol": "Apache",
            "amountSold": 2000.00
        },{
            "patrol": "Patrol X",
            "amountSold": 0.00
        },{
            "patrol": "Patrol Y",
            "amountSold": 0.00
        },{
            "patrol": "Patrol Z",
            "amountSold": 0.00
        }
    ],
    "userStats": {
        "patrol": "Bear",
        "name": "Fundraiser Admin",
        "isAdmin": true,
        "amountSold": 2000.00,
        "numOrders": 25
    }
};

/////////////////////////////////////////////
//
class SummaryInfo {
    private summaryResp_: any = mockSummaryResults;

    patrol(): string { return this.summaryResp_.userStats.patrol; }
    userName(): string { return this.summaryResp_.userStats.name; }
    isAdmin(): boolean { return this.summaryResp_.userStats.isAdmin===true; }
    totalAmountSold(): currency { return currency(this.summaryResp_.userStats.amountSold); }
    totalNumOrders(): number { return this.summaryResp_.userStats.numOrders; }
    totalTroopSold(): currency { return currency(this.summaryResp_.totalTroopAmountSold); }
    *patrolRankings(): Generator<[string, currency]> {
        for (const rank of this.summaryResp_.patrolRanking) {
            yield [rank.patrol, currency(rank.amountSold)];
        }
    }
}

/////////////////////////////////////////////
//
class OrderDb {
    private readonly fundraiserConfig_: any;
    private currentOrder_?: Order;

    constructor() {}

    /////////////////////////////////////////
    //
    setActiveOrder(order?: Order) {
        this.currentOrder_ = order;
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
    getOrderSummary(): Promise<SummaryInfo>  {
        return new Promise((resolve)=>{
            resolve(new SummaryInfo());
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
    getOrderList(): Promise<Array<OrderListItem<string>>> {

        const fieldNames:Array<string> = [
            "orderId",
            "firstName",
            "lastName",
            "addr1",
            "addr2",
            "phone",
            "email",
            "neighborhood",
            "amountTotal"
        ];

        return this.query({fields: fieldNames});
    }

    /////////////////////////////////////////
    //
    query(params: any): Promise<Array<any>> {
        return new Promise(async (resolve, reject)=>{
            try {
                auth.getAuthToken().then(async (authToken: string)=>{

                    params['orderOwner'] = auth.currentUser().getUsername();
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
                        alert(`HTTP Resp Error: ${resp.status}  ${errRespBody}`);
                        reject(null);
                    } else {
                        const ordersReturned: any = await resp.json();
                        //console.log(`OrdersDB Query Resp: ${JSON.stringify(ordersReturned)}`);
                        resolve(ordersReturned);
                    }
                });

                
            } catch(error) {
                console.error(error);
                alert("HTTP-Error: " + error);
                reject(null);
            }
        });
    }
}

const orderDb = new OrderDb()

export {orderDb, Order, OrdersForDeliveryDate, OrderListItem, SummaryInfo};
