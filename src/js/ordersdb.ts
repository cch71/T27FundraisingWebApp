

/////////////////////////////////////////////
//
interface OrderItemIf {
    totalDue: number;
    deliveryDate?: string
    kind: string;
}

/////////////////////////////////////////////
//
class NewOrder {
    name?: string;
    addr1?: string;
    addr2?: string;
    city?: string;
    state?: string;
    zip?: string;
    specialInstructions?: string;
    orderItems: Map<string, OrderItemIf>; //TODO: Don't want to lock in yet


    constructor() {
        this.state = "TX";
        this.orderItems = new Map<string, OrderItemIf>();
    }
}

/////////////////////////////////////////////
//
interface OrderIf {
    id: number;
    name: string;
    addr1: string;
    addr2?: string;
    city: string;
    state: string;
    zip: string;
    specialInstructions?: string;
    totalDue: number;
    orderItems?: Array<OrderItemIf>; //TODO: Don't want to lock in yet
}

/////////////////////////////////////////////
//
class OrderDb {
    private orders_: Array<OrderIf>;
    private deliveryDates_: any;
    private currentOrder_: NewOrder = new NewOrder();

    constructor() {
        const fakeOrder1 = {kind: 'mulch', bags: 20, toSpread: 20, totalDue: 20.40};
        const fakeOrder2 = {kind: 'donation', totalDue: 42.00};

        this.orders_ = [
            {
                id: 23, name: "MeMa Jons", addr1: '2020 I Rd',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: 0,
                orderItems: [fakeOrder1]
            },{
                id: 33, name: "La La", addr1: '1924 e ln',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: 0,
                orderItems: [fakeOrder1]
            },{
                id: 43, name: "The Royals", addr1: '221b Baker Street', addr2: "Loft C",
                city: "Round Rock", state: "TX", zip: "78641", totalDue: 0,
                orderItems: [fakeOrder1]
            },{
                id: 53, name: "Road Trippers", addr1: 'f323 cr',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: 0,
                orderItems: [fakeOrder2]
            },{
                id: 63, name: "I Known Know", addr1: '234 sfd',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: 0,
                orderItems: [fakeOrder1, fakeOrder2]
            },{
                id: 73, name: "The Jones", addr1: '567 wer',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: 0,
                orderItems: [fakeOrder1]
            },{
                id: 83, name: "The Blakes", addr1: '243 sdf',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: 0,
                orderItems: [fakeOrder1]
            },{
                id: 93, name: "The Bonds", addr1: '4564 sf',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: 0,
                orderItems: [fakeOrder1, fakeOrder2]
            },{
                id: 94, name: "Spectre", addr1: '243 ewre',
                city: "Round Rock", state: "TX", zip: "78641", totalDue: 0,
                orderItems: [fakeOrder1]
            }
        ];

        for (const order of this.orders_) {
            let totalDue = 0;
            if (order.orderItems) {
                for (const item of order.orderItems) {
                    totalDue += item.totalDue;
                }
            }
            order.totalDue = totalDue;
        }

        this.deliveryDates_ = [
            {
                date: '3/2/21',
                disabledDate: '3/4/21'
            },{
                date: '4/2/21',
                disabledDate: '4/4/21'
            }];

    }

    /////////////////////////////////////////
    //
    getCurrentFundraiserConfig(): any {
        //For Mulch
        return({
            pricePerBag: 5.00,
            pricePerSpread: 3.00
        });
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
        let totalDue = 0.0;
        let totalDonations = 0.0;
        let totalOrders = 0.0;
        for (const order of this.orders_) {
            totalDue += order.totalDue;
            if (order.orderItems) {
                for (const item of order.orderItems) {
                    if ('donation'===item.kind) {
                        totalDonations += item.totalDue;
                    } else {
                        totalOrders += item.totalDue;
                    }
                }
            }
        }

        return({
            totalOrderCost: totalDue,
            numOrders: this.orders_.length,
            totalDonations: totalDonations,
            totalOrders: totalOrders
        });
        
    }
    
    /////////////////////////////////////////
    //
    *deliveryDates(): IterableIterator<string> {
        
        for (let deliveryDate of this.deliveryDates_) {
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

export {orderDb, OrderIf, NewOrder, OrderItemIf};
