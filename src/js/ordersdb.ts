import currency from "currency.js"

/////////////////////////////////////////////
//
interface DeliverableOrderIf {
    totalDue: currency;
    deliveryDate?: string;
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
    neighborhood: string;
    specialInstructions?: string;
    cashPaid: currency;
    checkPaid: currency;
    deliverables: Map<string, DeliverableOrderIf>; //TODO: Don't want to lock in yet


    constructor(neighborhood: string) {
        this.state = "TX";
        this.cashPaid = currency(0.0);
        this.checkPaid = currency(0.0);
        this.neighborhood = neighborhood;
        this.deliverables = new Map<string, DeliverableOrderIf>();
    }
}

/////////////////////////////////////////////
//
interface OrderIf {
    id: number;
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
    private orders_: Array<OrderIf>;
    private readonly fundraiserConfig_: any;
    private currentOrder_: NewOrder;

    constructor() {
        this.fundraiserConfig_ = {
            description: "Mulch",
            kind: 'mulch',
            products: {
                bags: {
                    costDescription: "Price Per Bag",
                    cost: currency(5.00),
                    label: "Number of Bags"
                },
                spreading: {
                    costDescription: "Price Per Bag To Spread",
                    cost: currency(3.00),
                    label: "Bags to Spread"
                }
            },
            neighborhoods: ["Round Rock", "Forest Creek"],
            deliveryDates: [
                {
                date: '3/2/21',
                    disabledDate: '3/4/21'
                },{
                    date: '4/2/21',
                    disabledDate: '4/4/21'
                }
            ]
        };

        const defaultHood = this.fundraiserConfig_.neighborhoods[0];
        this.currentOrder_ = new NewOrder(defaultHood);

        const fakeOrder1 = {kind: 'mulch', bags: 20, toSpread: 20, totalDue: currency(20.40)};
        const fakeOrder2 = {kind: 'donation', totalDue: currency(42.00)};

        this.orders_ = [
            {
                id: 23, firstName: "MeMa", lastName:"Jons", addr1: '2020 I Rd',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: currency(0.0),
                orderItems: [fakeOrder1], neighborhood: defaultHood
            },{
                id: 33, firstName: "La", lastName: "La", addr1: '1924 e ln',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: currency(0.0),
                orderItems: [fakeOrder1], neighborhood: defaultHood
            },{
                id: 43, firstName: "The", lastName:"Royals", addr1: '221b Baker Street',
                addr2: "Loft C",
                city: "Round Rock", state: "TX", zip: "78641", totalDue: currency(0.0),
                orderItems: [fakeOrder1], neighborhood: defaultHood
            },{
                id: 53, firstName: "Road", lastName:"Trippers", addr1: 'f323 cr',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: currency(0.0),
                orderItems: [fakeOrder2], neighborhood: defaultHood
            },{
                id: 63, firstName: "I Known", lastName:"Know", addr1: '234 sfd',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: currency(0.0),
                orderItems: [fakeOrder1, fakeOrder2], neighborhood: defaultHood
            },{
                id: 73, firstName: "The", lastName: "Jones", addr1: '567 wer',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: currency(0.0),
                orderItems: [fakeOrder1], neighborhood: defaultHood
            },{
                id: 83, firstName: "The", lastName:"Blakes", addr1: '243 sdf',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: currency(0.0),
                orderItems: [fakeOrder1], neighborhood: defaultHood
            },{
                id: 93, firstName: "The", lastName:"Bonds", addr1: '4564 sf',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: currency(0.0),
                orderItems: [fakeOrder1, fakeOrder2], neighborhood: defaultHood
            },{
                id: 94, firstName: "a", lastName: "Spectre", addr1: '243 ewre',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: currency(0.0),
                orderItems: [fakeOrder1], neighborhood: defaultHood
            }
        ];

        for (const order of this.orders_) {
            let totalDue = currency(0.0);
            if (order.orderItems) {
                for (const item of order.orderItems) {
                    totalDue = totalDue.add(item.totalDue);
                }
            }
            order.totalDue = totalDue;
        }

    }

    /////////////////////////////////////////
    //
    getCurrentFundraiserConfig(): any {
        return this.fundraiserConfig_;
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
    *deliveryDates(): IterableIterator<string> {
        
        for (let deliveryDate of this.getCurrentFundraiserConfig().deliveryDates) {
            //if delivery date < disabledDate
            yield deliveryDate.date;
        }
        yield 'donation';
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
