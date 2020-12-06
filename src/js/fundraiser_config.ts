import awsConfig from "../config"


interface PriceBreak {
    gt: number;
    unitPrice: number;
}

interface Product {
    id: string;
    label: string;
    unitPrice: number;
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

    /////////////////////////////////////////
    //
    constructor(dlFrConfig: FundraiserConfigBase|null) {
        const getConfig = (): FundraiserConfigBase => {
            if (null === dlFrConfig) {
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
                    //console.error('Loading from session');
                    return JSON.parse(sessionFrConfig);
                } else {
                    console.error("Failed to load Session Fr Config");
                    throw ("Failed to load Session Fundraiser Config");
                }
            } else {
                return dlFrConfig;
            }
        };

        this.loadedFrConfig_ = getConfig();
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
    // return [id, date]
    *validDeliveryDates(): IterableIterator<[string,string]> {
        
        for (let deliveryDate of this.loadedFrConfig_.deliveryDates) {
            //if delivery date < disabledDate
            yield [deliveryDate.id, deliveryDate.date];
        }
        yield ['donation', 'Donation'];
    }

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
            const resp = await fetch(awsConfig.api.invokeUrl + '/getconfig', {
                method: 'post',
                headers: {
                    'Content-Type': 'application/json',
                    Authorization: authToken
                },
                body: JSON.stringify({})
            });

            if (!resp.ok) { // if HTTP-status is 200-299
                alert("HTTP Resp Error: " + resp.status);
                reject(null);
            } else {
                const loadedFrConfig: FundraiserConfigBase = await resp.json();
                console.log(`Fundraiser Config: ${JSON.stringify(loadedFrConfig)}`);

                window.sessionStorage.setItem('frConfig', JSON.stringify(loadedFrConfig));
                frConfig = new FundraiserConfig(loadedFrConfig);
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
        return new FundraiserConfig(null);
    }
    return frConfig;
}


export { FundraiserConfig, downloadFundraiserConfig, getFundraiserConfig};
