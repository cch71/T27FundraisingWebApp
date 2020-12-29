import awsConfig from "../config"


interface PriceBreak {
    gt: number;
    unitPrice: number;
}

interface Product {
    id: string;
    label: string;
    unitPrice: number;
	minUnits?: number;
    priceBreaks?: Array<PriceBreak>;
}

interface DeliveryDate {
    id: string;
    date: string;
    disabledDate: string;
}

interface FundraiserConfigBase {
    kind: string;
    description: string;
    products: Array<Product>;
    neighborhoods: Array<string>;
    deliveryDates: Array<DeliveryDate>;
}

/////////////////////////////////////////
//
class FundraiserConfig {
    private readonly loadedFrConfig_: FundraiserConfigBase;
    private readonly loadedPatrolMap_: any;
    private deliveryMap_: Record<string, string>|null = null;
    private usersMap_?: [[string, string]]|undefined;

    /////////////////////////////////////////
    //
    constructor(dlFrConfig?: FundraiserConfigBase, dlPatrolMap?: any) {
        const getConfig = (): FundraiserConfigBase => {
            if (!dlFrConfig) {
                if (typeof window === 'undefined')  {
                    return({
                        kind: '',
                        description: '',
                        products: [],
                        neighborhoods: [],
                        deliveryDates: []
                    });
                } // should only hit while building
                let sessionFrConfig = window.sessionStorage.getItem('frConfig');
                if (sessionFrConfig) {
                    console.log('Loading frConfig from session');
                    return JSON.parse(sessionFrConfig);
                } else {
                    console.error("Failed to load Session Fr Config");
                    throw new Error("Failed to load Session Fundraiser Config");
                }
            } else {
                return dlFrConfig;
            }
        };

        const getPatrolMap = ():any => {
            if (!dlPatrolMap) {
                if (typeof window === 'undefined')  { return {}; }
                let sessionPatrolMap = window.sessionStorage.getItem('patrolMap');
                if (sessionPatrolMap) {
                    console.log('Loading patrolMap from session');
                    return JSON.parse(sessionPatrolMap);
                } else {
                    console.error("Failed to load Session Patrol Map");
                    throw new Error("Failed to load Session Fundraiser Patrol Map");
                }
            } else {
                return dlPatrolMap;
            }
        };

        this.loadedFrConfig_ = getConfig();
        this.loadedPatrolMap_ = getPatrolMap();
    }

    /////////////////////////////////////////
    //
    kind(): string { return this.loadedFrConfig_.kind; }

    /////////////////////////////////////////
    //
    description(): string { return this.loadedFrConfig_.description; }

    /////////////////////////////////////////
    //
    neighborhoods(): Array<string> { return this.loadedFrConfig_.neighborhoods; }

    /////////////////////////////////////////
    //
    getUserNameFromId(uid: string): string {
        if ('fradmin' === uid) { return "Fundraiser Admin"; } //immutable admin id
        for (const [patrolName, names] of  Object.entries(this.loadedPatrolMap_)) {
            if (names.hasOwnProperty(uid)) {
                return names[uid]['name']
            }
        }
        return "Unknown";
    }

    /////////////////////////////////////////
    //
    *users(): Generator<[string, string]> {
        if (!this.usersMap_) {
            this.userMap_ = [];
            for (const namesObj of  Object.values(this.loadedPatrolMap_)) {
                for (const uid of  Object.keys(namesObj)) {
                    this.userMap_.push([uid, namesObj[uid]['name']]);
                }
            }
            this.userMap_.sort((a, b) => {
                return a[0] > b[0] ? 1 : a[0] < b[0] ? -1 : 0;
            });
        }
        for (const userInfo of this.userMap_) {
            yield userInfo;
        }
        yield ['fradmin', "Fundraiser Admin"]; //immutable admin id
    }

    /////////////////////////////////////////
    //)/*: Generator<>*/
    *products(): Generator<Product> {
        const oldProds = this.loadedFrConfig_.products;
        for (const product of this.loadedFrConfig_.products) {
            if (!product.hasOwnProperty('priceBreaks')) {
                product['priceBreaks'] = [];
            }
            yield product;
        }
    }

    /////////////////////////////////////////
    //
    private loadDeliveryMap() {
        if(null===this.deliveryMap_) {
            this.deliveryMap_ = {};
            for (const deliveryDate of this.loadedFrConfig_.deliveryDates) {
                this.deliveryMap_[deliveryDate.id] = deliveryDate.date;
            }
        }
    }

    /////////////////////////////////////////
    //
    isEditableDeliveryDate(id?: string): boolean {
        if (!id) { return true; }
        this.loadDeliveryMap();
        const deliveryEntry = this.deliveryMap_[id];
        try {
            const deliveryDate = Date.parse(deliveryEntry.date);
            const nowDate = Date.now();
            if (nowDate >= deliveryDate) { return false; }

            if (deliveryEntry.disabledDate) {
                const disabledDate = Date.parse(deliveryEntry.disabledDate);
                if (nowDate >= disabledDate) {
                    return false;
                }
            }

        }catch(err) {
            console.error(`Failed handling valid delivery date ${err}`);
        }
        return true;
    }

    /////////////////////////////////////////
    // return [id, date]
    *validDeliveryDates(): IterableIterator<[string,string]> {
        for (let deliveryDate of this.loadedFrConfig_.deliveryDates) {
            //if delivery date < disabledDate
            yield [deliveryDate.id, deliveryDate.date];
        }
        yield ['donation', 'Donation'];
    }

    /////////////////////////////////////////
    //
    deliveryDateFromId(id: string): string {
        this.loadDeliveryMap();
        return this.deliveryMap_[id];
    }

    /////////////////////////////////////////
    //
    numDeliveryDates(): number {
        return this.loadedFrConfig_.deliveryDates.length;
    }

}


/////////////////////////////////////////
//
let frConfig: FundraiserConfig|null = null;

/////////////////////////////////////////
//
function downloadFundraiserConfig(authToken: string): Promise<FundraiserConfig | null> {
    return new Promise(async (resolve, reject)=>{
        try {
            console.log("Downloading Fundraiser Configs");
            const getConfigPromise = fetch(awsConfig.api.invokeUrl + '/getconfig', {
                method: 'post',
                headers: {
                    'Content-Type': 'application/json',
                    Authorization: authToken
                }
            });

            const patrolMapPromise = fetch(awsConfig.api.invokeUrl + '/patrolmap', {
                method: 'post',
                headers: {
                    'Content-Type': 'application/json',
                    Authorization: authToken
                }
            });

            const [frConfigResp, patrolMapResp] = await Promise.all([getConfigPromise, patrolMapPromise]);

            if (!frConfigResp.ok) { // if HTTP-status is 200-299
                alert("HTTP Fundraiser Config Resp Error: " + frConfigResp.status);
                reject(null);
            } else if(!patrolMapResp.ok) {
                alert("HTTP Fundraiser PatrolMap Resp Error: " + patrolMapResp.status);
                reject(null);
            } else {
                const loadedFrConfig: FundraiserConfigBase = await frConfigResp.json();
                console.log(`Fundraiser Config: ${JSON.stringify(loadedFrConfig)}`);
                const loadedPatrolMap: any = await patrolMapResp.json();
                //console.log(`Patrol Map: ${JSON.stringify(loadedPatrolMap)}`);

                window.sessionStorage.setItem('frConfig', JSON.stringify(loadedFrConfig));
                window.sessionStorage.setItem('patrolMap', JSON.stringify(loadedPatrolMap));
                frConfig = new FundraiserConfig(loadedFrConfig, loadedPatrolMap);
                resolve(frConfig);
            }
        } catch(error) {
            console.error(error);
            alert("HTTP-Error: " + error);
            reject(null);
        }
    });
}

/////////////////////////////////////////
//
function getFundraiserConfig(): FundraiserConfig {
    if (null===frConfig) {
        console.log("Loading Default Config");
        return new FundraiserConfig();
    }
    return frConfig;
}


export { FundraiserConfig, downloadFundraiserConfig, getFundraiserConfig};
