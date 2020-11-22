import currency from "currency.js"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"

/////////////////////////////////////////////
//
interface DeliverableOrderIf {
    totalDue: currency;
    deliveryId?: string;
    kind: string;
    items?: Map<string, number> //productId, numOrders
}

/////////////////////////////////////////////
//
class NewOrder {
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
    specialInstructions?: string;
    cashPaid: currency;
    checkPaid: currency;
    deliverables: Map<string, DeliverableOrderIf>; //TODO: Don't want to lock in yet


    constructor() {
        this.state = "TX";
        this.cashPaid = currency(0.0);
        this.checkPaid = currency(0.0);
        this.deliverables = new Map<string, DeliverableOrderIf>();
    }
}

/////////////////////////////////////////////
//
interface OrderIf {
    orderOwner: string;
    orderId: string;
    firstName: string;
    lastName: string;
    phone?: string;
    email?: string;
    addr1: string;
    addr2?: string;
    city: string;
    state: string;
    zip: string;
    neighborhood: string;
    specialInstructions?: string;
    cashPaid?: currency;
    checkPaid?: currency;
    totalDue: currency;
    orderItems?: Array<DeliverableOrderIf>; //TODO: Don't want to lock in yet
}

/////////////////////////////////////////////
//
class OrderDb {
    private orders_: Array<OrderIf> = new Array<OrderIf>();
    private readonly fundraiserConfig_: any;
    private currentOrder_: NewOrder;

    constructor() {
        this.currentOrder_ = new NewOrder();
    }

    /////////////////////////////////////////
    //
    setCurrentOrder(order: NewOrder) {
        this.currentOrder_ = order;
    }

    /////////////////////////////////////////
    //
    getCurrentOrder(): NewOrder {
        return this.currentOrder_;
    }

    /////////////////////////////////////////
    //
    // Todo need to define summary type
    getOrderSummary(): any  {
        // Todo save the summary so we don't keep calc
        let totalDue = currency(0.0);
        let totalDonationsAmmount = currency(0.0);
        let totalOrdersAmount = currency(0.0);
        for (const order of this.orders_) {
            totalDue = totalDue.add(order.totalDue);
            if (order.orderItems) {
                for (const item of order.orderItems) {
                    if ('donation'===item.kind) {
                        totalDonationsAmmount = totalDonationsAmmount.add(item.totalDue);
                    } else {
                        totalOrdersAmount = totalOrdersAmount.add(item.totalDue);
                    }
                }
            }
        }
        const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });
        
        return({
            totalOrderCost: USD(totalDue).format(),
            numOrders: this.orders_.length,
            totalDonations: USD(totalDonationsAmmount).format(),
            totalOrders: USD(totalOrdersAmount).format()
        });
        
    }
        
    /////////////////////////////////////////
    //
    *orders(): IterableIterator<OrderIf> {
        for (let order of this.orders_) {
            yield order;
        }
    }
}

const orderDb = new OrderDb()

export {orderDb, OrderIf, NewOrder, DeliverableOrderIf};
