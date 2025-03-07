use rand::Rng;
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::VecDeque;
//Here we are storing the order as struct to matching scenarios and sorting the prices and time
//we can have epoch time in place of timestamp which would give exact time.
#[derive(Debug, Clone)]
struct Order {
    id: usize,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

//Here am using double ended queue to add and remove orders
//why double ended queue? we can easily extract the higher priority order in less time as each and every we add the order we are going to sort.(similar to pq discussed in the interview)
#[derive(Debug)]
struct OrderBook {
    buy_orders: VecDeque<Order>,
    sell_orders: VecDeque<Order>,
    next_order_id: usize,
    current_price: f64,
}

impl OrderBook {
    //Initialisation of the order book ,creating 2 queues and initalizing the order id with 1
    async fn new() -> Self {
        let current_price = Self::fetch_current_price().await.unwrap_or(45700.0);
        Self {
            buy_orders: VecDeque::new(),
            sell_orders: VecDeque::new(),
            next_order_id: 1,
            current_price,
        }
    }

    async fn fetch_current_price() -> Option<f64> {
        let url = "https://api.binance.com/api/v3/ticker/price";
        if let Ok(response) = reqwest::get(url).await {
            if let Ok(result) = response.text().await {
                if let Ok(json_data) = serde_json::from_str::<Value>(&result) {
                    if let Some(array) = json_data.as_array() {
                        for obj in array {
                            if let Some(symbol) = obj.get("symbol") {
                                if symbol == "BTCUSDT" {
                                    if let Some(price) = obj.get("price") {
                                        return price.as_str().and_then(|p| p.parse::<f64>().ok());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
    //Creating the struct based on the price ,quantity,time stamp
    fn add_order(&mut self, price: f64, quantity: f64, is_buy: bool, timestamp: u64) {
        let order = Order {
            id: self.next_order_id,
            price,
            quantity,
            timestamp,
        };
        self.next_order_id += 1;
        //Here we will be adding the orders into the queue and sorting will happen based on the prices and if prices are equal then we will sort with time
        //logic::Buy order prices are sorted in descending order and sell order prices are sorted in ascending order which makes orders to fill faster
        if is_buy {
            self.buy_orders.push_back(order);
            self.buy_orders.make_contiguous().sort_by(|a, b| {
                match b.price.partial_cmp(&a.price).unwrap() {
                    Ordering::Equal => a.timestamp.cmp(&b.timestamp),
                    other => other,
                }
            });
        } else {
            self.sell_orders.push_back(order);
            self.sell_orders.make_contiguous().sort_by(|a, b| {
                match a.price.partial_cmp(&b.price).unwrap() {
                    Ordering::Equal => a.timestamp.cmp(&b.timestamp),
                    other => other,
                }
            });
        }
        //Each and every time you added a order we have to check the order book and match if any
        self.match_orders();
    }

    //This is to modify the order ,we can further increase the functionality to change price of the order based upon the quantity and price
    fn modify_order(&mut self, order_id: usize, new_quantity: f64) {
        for order in self
            .buy_orders
            .iter_mut()
            .chain(self.sell_orders.iter_mut())
        {
            if order.id == order_id {
                order.quantity = new_quantity;
                break;
            }
        }
    }
    //here we will be matching the best buy order for best sell order if and only if the buy order price is greater than sell order price,
    //Logic for matching and partial  matching is done and we can also increase or decrease current price based upon this order matchings
    //min(buyorder.quantity,sellorder.quantity) gives the exact amount of order that can match
    fn match_orders(&mut self) {
        while let (Some(mut buy_order), Some(mut sell_order)) = (
            self.buy_orders.front().cloned(),
            self.sell_orders.front().cloned(),
        ) {
            if buy_order.price >= sell_order.price {
                let transaction_quantity = buy_order.quantity.min(sell_order.quantity);
                println!(
                    "Matched: Buy Order {} and Sell Order {} at price {} for quantity {}",
                    buy_order.id, sell_order.id, sell_order.price, transaction_quantity
                );

                if buy_order.quantity > transaction_quantity {
                    self.buy_orders.front_mut().unwrap().quantity -= transaction_quantity;
                } else {
                    self.buy_orders.pop_front();
                }

                if sell_order.quantity > transaction_quantity {
                    self.sell_orders.front_mut().unwrap().quantity -= transaction_quantity;
                } else {
                    self.sell_orders.pop_front();
                }
            } else {
                break;
            }
        }
    }
}

//for generating random bulk orders
fn create_bulk_orders(order_book: &mut OrderBook, num_orders: usize) {
    let mut rng = rand::thread_rng();
    let base_price = 45700.0;
    for i in 0..num_orders {
        let price = base_price + rng.gen_range(-100..100) as f64;
        let quantity = rng.gen_range(0.1..5.0);
        let is_buy = rng.gen_bool(0.5);
        order_book.add_order(price, quantity, is_buy, i as u64);
    }
}
#[tokio::main]
async fn main() {
    let mut order_book = OrderBook::new().await;

    // order_book.add_order(101.0, 5.0, true, 1);
    // order_book.add_order(100.5, 3.0, true, 2);
    // order_book.add_order(100.0, 4.0, false, 3);
    // order_book.add_order(99.5, 2.0, false, 4);

    // order_book.add_order(45700.0, 1.0, true, 1);
    // order_book.add_order(45650.0, 2.5, true, 2);
    // order_book.add_order(45600.0, 1.0, true, 1);
    // order_book.add_order(45710.0, 1.0, false, 2);
    // order_book.add_order(45720.0, 1.5, false, 3);
    // order_book.add_order(45750.0, 2.0, false, 4);
    create_bulk_orders(&mut order_book, 10);
    println!("Remaining Buy Orders: {:?}", order_book.buy_orders);
    println!("Remaining Sell Orders: {:?}", order_book.sell_orders);
    println!("Current BTC/USDT Price: {}", order_book.current_price);
}
