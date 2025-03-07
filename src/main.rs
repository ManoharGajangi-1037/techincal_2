use std::collections::VecDeque;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct Order {
    id: usize,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct OrderBook {
    buy_orders: VecDeque<Order>,
    sell_orders: VecDeque<Order>,
    next_order_id: usize,
}

impl OrderBook {
    fn new() -> Self {
        Self {
            buy_orders: VecDeque::new(),
            sell_orders: VecDeque::new(),
            next_order_id: 1,
        }
    }

    fn add_order(&mut self, price: f64, quantity: f64, is_buy: bool, timestamp: u64) {
        let order = Order {
            id: self.next_order_id,
            price,
            quantity,
            timestamp,
        };
        self.next_order_id += 1;

        if is_buy {
            self.buy_orders.push_back(order);
            self.buy_orders.make_contiguous().sort_by(|a, b| match b.price.partial_cmp(&a.price).unwrap() {
                Ordering::Equal => a.timestamp.cmp(&b.timestamp),
                other => other,
            });
        } else {
            self.sell_orders.push_back(order);
            self.sell_orders.make_contiguous().sort_by(|a, b| match a.price.partial_cmp(&b.price).unwrap() {
                Ordering::Equal => a.timestamp.cmp(&b.timestamp),
                other => other,
            });
        }

        self.match_orders();
    }

    fn modify_order(&mut self, order_id: usize, new_quantity: f64) {
        for order in self.buy_orders.iter_mut().chain(self.sell_orders.iter_mut()) {
            if order.id == order_id {
                order.quantity = new_quantity;
                break;
            }
        }
    }

    fn match_orders(&mut self) {
        while let (Some(mut buy_order), Some(mut sell_order)) = (self.buy_orders.front().cloned(), self.sell_orders.front().cloned()) {
            if buy_order.price >= sell_order.price {
                let transaction_quantity = buy_order.quantity.min(sell_order.quantity);
                println!("Matched: Buy Order {} and Sell Order {} at price {} for quantity {}", buy_order.id, sell_order.id, sell_order.price, transaction_quantity);
                
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

fn main() {
    let mut order_book = OrderBook::new();

    // order_book.add_order(101.0, 5.0, true, 1);
    // order_book.add_order(100.5, 3.0, true, 2);
    // order_book.add_order(100.0, 4.0, false, 3);
    // order_book.add_order(99.5, 2.0, false, 4);
    
    order_book.add_order(45700.0, 1.0, true, 1);
    order_book.add_order(45650.0, 2.5, true, 2);
    order_book.add_order(45600.0, 1.0, true, 1);
    order_book.add_order(45710.0, 1.0, false, 2);
    order_book.add_order(45720.0, 1.5, false, 3);
    order_book.add_order(45750.0, 2.0, false, 4);
    println!("Remaining Buy Orders: {:?}", order_book.buy_orders);
    println!("Remaining Sell Orders: {:?}", order_book.sell_orders);
}
