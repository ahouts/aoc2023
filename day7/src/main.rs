use anyhow::{anyhow, Context, Result};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Add;
use z3::ast::{Ast, Bool, Int};
use z3::SatResult;

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
enum Card {
    Ace,
    King,
    Queen,
    Jack,
    Ten,
    Nine,
    Eight,
    Seven,
    Six,
    Five,
    Four,
    Three,
    Two,
}

impl Card {
    fn joker_value(self) -> u8 {
        match self {
            Card::Ace => 1,
            Card::King => 2,
            Card::Queen => 3,
            Card::Jack => 13,
            Card::Ten => 4,
            Card::Nine => 5,
            Card::Eight => 6,
            Card::Seven => 7,
            Card::Six => 8,
            Card::Five => 9,
            Card::Four => 10,
            Card::Three => 11,
            Card::Two => 12,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
struct Hand {
    cards: [Card; 5],
    bid: usize,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
enum HandKind {
    FiveOfAKind,
    FourOfAKind,
    FullHouse,
    ThreeOfAKind,
    TwoPair,
    OnePair,
    HighCard,
}

impl Hand {
    fn kind(self) -> HandKind {
        let mut hand = HashMap::new();
        for card in self.cards {
            *hand.entry(card).or_default() += 1;
        }
        match hand.values().max().unwrap() {
            5 => HandKind::FiveOfAKind,
            4 => HandKind::FourOfAKind,
            3 => {
                if *hand.values().filter(|v| **v != 3).max().unwrap() == 2 {
                    HandKind::FullHouse
                } else {
                    HandKind::ThreeOfAKind
                }
            }
            2 => {
                if hand.values().filter(|v| **v == 2).count() == 2 {
                    HandKind::TwoPair
                } else {
                    HandKind::OnePair
                }
            }
            _ => HandKind::HighCard,
        }
    }

    fn joker_kind(self) -> HandKind {
        fn any_total<'a>(
            ctx: &'a z3::Context,
            totals: &'a [Int<'a>],
            cond: impl Fn(&'a Int<'a>) -> Bool<'a>,
        ) -> Bool<'a> {
            let f = Bool::from_bool(&ctx, false);
            let mut any = f.clone();
            for total in totals {
                let res = cond(total);
                any = any._eq(&f).ite(&res._eq(&f), &f).not();
            }
            any
        }

        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);
        let opt = z3::Optimize::new(&ctx);

        let two = Int::from_i64(&ctx, 2);
        let one = Int::from_i64(&ctx, 1);
        let zero = Int::from_i64(&ctx, 0);
        let twelve = Int::from_i64(&ctx, 12);

        let hand = (0..5)
            .map(|idx| Int::new_const(&ctx, format!("h{}", idx).as_str()))
            .collect::<Vec<Int>>();
        for (var, card) in hand.iter().zip(self.cards) {
            if card == Card::Jack {
                opt.assert(&var.ge(&one));
                opt.assert(&var.le(&twelve));
            } else {
                opt.assert(&var._eq(&Int::from_i64(&ctx, card.joker_value() as i64)));
            }
        }
        let totals: Vec<Int> = (1..=12)
            .map(|value| {
                let value = Int::from_i64(&ctx, value);
                let mut total = zero.clone();
                for var in hand.iter() {
                    total = total.add(&var._eq(&value).ite(&one, &zero));
                }
                total
            })
            .collect();

        let f = Bool::from_bool(&ctx, false);

        let five_of_a_kind = any_total(&ctx, totals.as_slice(), |total| {
            total._eq(&Int::from_i64(&ctx, 5))
        })
        .ite(&Int::from_i64(&ctx, 1 << 6), &zero);
        let four_of_a_kind = any_total(&ctx, totals.as_slice(), |total| {
            total._eq(&Int::from_i64(&ctx, 4))
        })
        .ite(&Int::from_i64(&ctx, 1 << 5), &zero);
        let full_house = any_total(&ctx, totals.as_slice(), |total| {
            total._eq(&Int::from_i64(&ctx, 3))
        })
        .ite(
            &any_total(&ctx, totals.as_slice(), |total| {
                total._eq(&Int::from_i64(&ctx, 2))
            }),
            &f,
        )
        .ite(&Int::from_i64(&ctx, 1 << 4), &zero);
        let three_of_a_kind = any_total(&ctx, totals.as_slice(), |total| {
            total._eq(&Int::from_i64(&ctx, 3))
        })
        .ite(&Int::from_i64(&ctx, 1 << 3), &zero);

        let two_pair = {
            let mut sum = zero.clone();
            for total in &totals {
                sum = sum.add(&total._eq(&two).ite(&one, &zero));
            }
            sum._eq(&two)
        }
        .ite(&Int::from_i64(&ctx, 1 << 2), &zero);

        let pair = any_total(&ctx, totals.as_slice(), |total| {
            total._eq(&Int::from_i64(&ctx, 2))
        })
        .ite(&Int::from_i64(&ctx, 1 << 1), &zero);

        let score = five_of_a_kind
            .add(&four_of_a_kind)
            .add(&full_house)
            .add(&three_of_a_kind)
            .add(&two_pair)
            .add(&pair);

        opt.maximize(&score);
        if opt.check(&[]) == SatResult::Unsat {
            panic!();
        }
        if let Some(ans) = opt.get_model() {
            if let Some(res) = ans.eval(&score, true) {
                if let Some(value) = res.as_i64() {
                    if value >= 1 << 6 {
                        return HandKind::FiveOfAKind;
                    }
                    if value >= 1 << 5 {
                        return HandKind::FourOfAKind;
                    }
                    if value >= 1 << 4 {
                        return HandKind::FullHouse;
                    }
                    if value >= 1 << 3 {
                        return HandKind::ThreeOfAKind;
                    }
                    if value >= 1 << 2 {
                        return HandKind::TwoPair;
                    }
                    if value >= 1 << 1 {
                        return HandKind::OnePair;
                    }
                    return HandKind::HighCard;
                }
            }
        }

        panic!()
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.kind()
            .cmp(&other.kind())
            .then_with(|| self.cards.cmp(&other.cards))
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_hands() -> Result<Vec<Hand>> {
    let mut hands = Vec::new();
    for result in
        BufReader::new(File::open("day7/input.txt").context("error opening input")?).lines()
    {
        let line = result.context("error reading input")?;
        let (cards_text, bid_text) = line.split_at(5);
        let mut cards = [Card::Ace; 5];
        for (i, card) in cards_text.chars().enumerate() {
            cards[i] = match card {
                'A' => Card::Ace,
                'K' => Card::King,
                'Q' => Card::Queen,
                'J' => Card::Jack,
                'T' => Card::Ten,
                '9' => Card::Nine,
                '8' => Card::Eight,
                '7' => Card::Seven,
                '6' => Card::Six,
                '5' => Card::Five,
                '4' => Card::Four,
                '3' => Card::Three,
                '2' => Card::Two,
                unexpected => return Err(anyhow!("unknown card {unexpected:?}")),
            }
        }
        let bid = bid_text.trim().parse().context("error parsing bid")?;
        hands.push(Hand { cards, bid })
    }
    Ok(hands)
}

fn main() -> Result<()> {
    let hands = parse_hands()?;
    println!("part1: {}", part1(hands.as_slice())?);
    println!("part2: {}", part2(hands.as_slice())?);
    Ok(())
}

fn part1(hands: &[Hand]) -> Result<usize> {
    let mut hands = hands.to_vec();
    hands.sort();
    hands.reverse();
    Ok(hands
        .into_iter()
        .enumerate()
        .map(|(pos, hand)| hand.bid * (1 + pos))
        .sum())
}

fn part2(hands: &[Hand]) -> Result<usize> {
    let mut hands = hands
        .iter()
        .map(|hand| {
            (
                (hand.joker_kind(), hand.cards.map(|card| card.joker_value())),
                hand,
            )
        })
        .collect::<Vec<_>>();
    hands.sort_by_key(|(key, _)| *key);
    hands.reverse();
    for (_, hand) in &hands {
        println!("{:?} - {:?}", hand.cards, hand.joker_kind())
    }
    Ok(hands
        .into_iter()
        .enumerate()
        .map(|(pos, (_, hand))| hand.bid * (1 + pos))
        .sum())
}
