
use std::collections::{HashMap};
use std::marker::PhantomData;

use crate::tokenizer::{EventIterator};
use crate::tokenizer::{Reader};


use super::YamlToken;

pub struct YamlParser<'a, R, RB, I, TAG = ()> {
    pub(crate) iterator: EventIterator<'a, R, RB, I>,
    pub(crate) _map: HashMap<String, &'a YamlToken<'a, TAG>>,

}

