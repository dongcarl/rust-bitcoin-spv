//
// Copyright 2018-2019 Tamas Blummer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
//!
//! # Serve general header and block requests
//!

use bitcoin::{
    BitcoinHash,
    network::message::NetworkMessage,
    network::message_blockdata::{GetHeadersMessage,GetBlocksMessage, InvType, Inventory},
    blockdata::block::{Block, LoneBlockHeader},
    util::hash::Sha256dHash,
    consensus::encode::VarInt
};
use blockfilter::{COIN_FILTER, SCRIPT_FILTER};
use chaindb::SharedChainDB;
use chaindb::StoredFilter;
use error::SPVError;
use p2p::{P2PControl, P2PControlSender, PeerId, PeerMessage, PeerMessageReceiver, PeerMessageSender};
use std::{
    sync::mpsc,
    thread
};

pub struct BlockServer {
    p2p: P2PControlSender,
    chaindb: SharedChainDB,
}

// channel size
const BACK_PRESSURE: usize = 10;

impl BlockServer {
    pub fn new(chaindb: SharedChainDB, p2p: P2PControlSender) -> PeerMessageSender {
        let (sender, receiver) = mpsc::sync_channel(BACK_PRESSURE);

        let mut block_server = BlockServer { chaindb, p2p };

        thread::spawn(move || { block_server.run(receiver) });

        PeerMessageSender::new(sender)
    }

    fn run(&mut self, receiver: PeerMessageReceiver) {
        while let Ok(msg) = receiver.recv() {
            if let Err(e) = match msg {
                PeerMessage::Message(pid, msg) => {
                    match msg {
                        NetworkMessage::GetHeaders(get) => self.get_headers(pid, get),
                        NetworkMessage::GetBlocks(get) => self.get_blocks(pid, get),
                        NetworkMessage::GetData(get) => self.get_data(pid, get),
                        _ => { Ok(()) }
                    }
                }
                _ => {Ok(())}
            } {
                error!("Error processing headers: {}", e);
            }
        }
        panic!("Block server thread failed.");
    }

    fn get_headers(&self, peer: PeerId, get: GetHeadersMessage) -> Result<(), SPVError> {
        let chaindb = self.chaindb.read().unwrap();
        for locator in get.locator_hashes.iter () {
            if chaindb.is_on_trunk(locator) {
                let mut headers = Vec::with_capacity(2000);
                for block_id in chaindb.iter_to_tip(locator).take(2000) {
                    headers.push(LoneBlockHeader{header: chaindb.get_header(&block_id).unwrap().header, tx_count: VarInt(0)})
                }
                if headers.len () > 0 {
                    self.p2p.send(P2PControl::Send(peer, NetworkMessage::Headers(headers)));
                }
                break;
            }
        }
        Ok(())
    }

    fn get_blocks(&self, peer: PeerId, get: GetBlocksMessage) -> Result<(), SPVError> {
        let chaindb = self.chaindb.read().unwrap();
        for locator in get.locator_hashes.iter () {
            if chaindb.is_on_trunk(locator) {
                for block_id in chaindb.iter_to_tip(locator).take(500) {
                    let header = chaindb.get_header(&block_id).unwrap();
                    if let Some(pref) = header.block {
                        let block = chaindb.fetch_block_by_ref(pref)?;
                        self.p2p.send(P2PControl::Send(peer, NetworkMessage::Block(Block{header: header.header, txdata: block.txdata})));
                    }
                }
                break;
            }
        }
        Ok(())
    }

    fn get_data(&self, peer: PeerId, get: Vec<Inventory>) -> Result<(), SPVError> {
        let chaindb = self.chaindb.read().unwrap();
        for inv in get {
            if inv.inv_type == InvType::WitnessBlock {
                if let Some(header) = chaindb.get_header(&inv.hash) {
                    if let Some(pref) = header.block {
                        let block = chaindb.fetch_block_by_ref(pref)?;
                        self.p2p.send(P2PControl::Send(peer, NetworkMessage::Block(Block{header: header.header, txdata: block.txdata})));
                    }
                }
            }
        }
        Ok(())
    }
}