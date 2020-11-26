import currency from "currency.js"
import awsConfig from "../config"

interface Product<T> {
    id: string,
    label: string,
    costDescription: string,
    cost: T,
}

interface DeliveryDate {
    id: string;
    date: string;
    disabledDate: string;
}

interface FundraiserConfigBase<T> {
    kind: string;
    description: string;
    products: Array<Product<T>>;
    neighborhoods: Array<string>;
    deliveryDates: Array<DeliveryDate>;
}

/////////////////////////////////////////
//
class FundraiserConfig {
    private readonly loadedFrConfig_: FundraiserConfigBase<number>;

    /////////////////////////////////////////
    //
    constructor(dlFrConfig: FundraiserConfigBase<number>|null) {
        const getConfig = (): FundraiserConfigBase<number> => {
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
    *products(): Generator<Product<currency>> {
        const oldProds = this.loadedFrConfig_.products;
        for (let x=0; x<oldProds.length; x++) {
            const oldProd = oldProds[x];
            const newProd: Product<currency> = {
                id: oldProd.id,
                cost: currency(oldProd.cost),
                label: oldProd.label,
                costDescription: oldProd.costDescription
            };
            console.log(`Generating: ${JSON.stringify(newProd)}`);
            yield newProd;
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
                const loadedFrConfig: FundraiserConfigBase<number> = await resp.json();
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
