

/////////////////////////////////////////////
//
interface OrderItemIf {
    totalDue: number;
    kind: String;
}


/////////////////////////////////////////////
//
interface OrderIf {
    id: number;
    name: String;
    addr1: String;
    addr2?: String;
    city: String;
    state: String;
    zip: String;
    specialInstructions?: String;
    totalDue: number;
    orderItems?: Array<OrderItemIf>; //TODO: Don't want to lock in yet
}

/////////////////////////////////////////////
//
class OrderDb {
    private db_: Array<OrderIf>;

    constructor() {
        const fakeOrder1 = {kind: 'mulch', bags: 20, toSpread: 20, totalDue: 20.40};
        const fakeOrder2 = {kind: 'donation', totalDue: 42.00};

        this.db_ = [
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

        for (const order of this.db_) {
            let totalDue = 0;
            if (order.orderItems) {
                for (const item of order.orderItems) {
                    totalDue += item.totalDue;
                }
            }
            order.totalDue = totalDue;
        }

    }

    /////////////////////////////////////////
    //
    getOrders(): Array<OrderIf> {

        return this.db_;
    }
}

const orderDb = new OrderDb()

export {orderDb, OrderIf};
