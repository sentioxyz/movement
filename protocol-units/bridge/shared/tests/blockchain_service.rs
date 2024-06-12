use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};

use bridge_shared::blockchain_service::{BlockchainEvent, BlockchainService};
use bridge_shared::bridge_contracts::{
	BridgeContractCounterparty, BridgeContractInitiator, BridgeContractResult,
};
use bridge_shared::bridge_monitoring::{
	BridgeContractCounterpartyEvent, BridgeContractCounterpartyMonitoring,
	BridgeContractInitiatorEvent, BridgeContractInitiatorMonitoring,
};
use bridge_shared::types::{BridgeTransferDetails, BridgeTransferId};

struct MockInitiatorMonitoring {
	events: Vec<BridgeContractInitiatorEvent<&'static str, &'static str>>,
}

struct MockBlockchainService {
	initiator_contract: MockInitiatorContract,
	initiator_monitoring: MockInitiatorMonitoring,
	counterparty_contract: MockCounterpartyContract,
	counterparty_monitoring: MockCounterpartyMonitoring,
}

impl BlockchainService for MockBlockchainService {
	type Address = &'static str;
	type Hash = &'static str;

	type InitiatorContract = MockInitiatorContract;
	type InitiatorMonitoring = MockInitiatorMonitoring;

	type CounterpartyContract = MockCounterpartyContract;
	type CounterpartyMonitoring = MockCounterpartyMonitoring;

	fn initiator_contract(&self) -> &Self::InitiatorContract {
		&self.initiator_contract
	}

	fn initiator_monitoring(&self) -> &Self::InitiatorMonitoring {
		&self.initiator_monitoring
	}

	fn counterparty_contract(&self) -> &Self::CounterpartyContract {
		&self.counterparty_contract
	}

	fn counterparty_monitoring(&self) -> &Self::CounterpartyMonitoring {
		&self.counterparty_monitoring
	}
}

impl Stream for MockBlockchainService {
	type Item = BlockchainEvent<&'static str, &'static str>;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let this = self.get_mut();

		// let initiator_monitoring = Pin::new(&mut this.initiator_monitoring);
		// let counterparty_monitoring = Pin::new(&mut this.counterparty_monitoring);

		match (
			this.initiator_monitoring.poll_next_unpin(cx),
			this.counterparty_monitoring.poll_next_unpin(cx),
		) {
			(Poll::Ready(Some(event)), _) => {
				Poll::Ready(Some(BlockchainEvent::InitiatorEvent(event)))
			}
			(_, Poll::Ready(Some(event))) => {
				Poll::Ready(Some(BlockchainEvent::CounterpartyEvent(event)))
			}
			_ => Poll::Pending,
		}
	}
}

impl Stream for MockInitiatorMonitoring {
	type Item = BridgeContractInitiatorEvent<&'static str, &'static str>;

	fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let this = self.get_mut();
		if let Some(event) = this.events.pop() {
			Poll::Ready(Some(event))
		} else {
			Poll::Pending
		}
	}
}

impl BridgeContractInitiatorMonitoring<&'static str, &'static str> for MockInitiatorMonitoring {}

struct MockCounterpartyMonitoring;

impl Stream for MockCounterpartyMonitoring {
	type Item = BridgeContractCounterpartyEvent<&'static str, &'static str>;

	fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		Poll::Pending
	}
}

impl BridgeContractCounterpartyMonitoring<&'static str, &'static str>
	for MockCounterpartyMonitoring
{
}

struct MockInitiatorContract;

#[async_trait::async_trait]
impl BridgeContractInitiator<&'static str, &'static str> for MockInitiatorContract {
	async fn initiate_bridge_transfer(
		&self,
		_initiator_address: &'static str,
		_recipient_address: &'static str,
		_hash_lock: &'static str,
		_time_lock: u64,
		_amount: u64,
	) -> BridgeContractResult<()> {
		Ok(())
	}

	async fn complete_bridge_transfer<S: Send>(
		&self,
		_bridge_transfer_id: BridgeTransferId<&'static str>,
		_secret: S,
	) -> BridgeContractResult<()> {
		Ok(())
	}

	async fn refund_bridge_transfer(
		&self,
		_bridge_transfer_id: BridgeTransferId<&'static str>,
	) -> BridgeContractResult<()> {
		Ok(())
	}

	async fn get_bridge_transfer_details(
		&self,
		_bridge_transfer_id: BridgeTransferId<&'static str>,
	) -> BridgeContractResult<Option<BridgeTransferDetails<&'static str, &'static str>>> {
		Ok(None)
	}
}

struct MockCounterpartyContract;

#[async_trait::async_trait]
impl BridgeContractCounterparty<&'static str, &'static str> for MockCounterpartyContract {
	async fn lock_bridge_transfer_assets(
		&self,
		_bridge_transfer_id: BridgeTransferId<&'static str>,
		_hash_lock: &'static str,
		_time_lock: u64,
		_recipient: &'static str,
		_amount: u64,
	) -> bool {
		true
	}

	async fn complete_bridge_transfer<S: Send>(
		&self,
		_bridge_transfer_id: &'static str,
		_secret: S,
	) -> BridgeContractResult<()> {
		Ok(())
	}

	async fn abort_bridge_transfer(
		&self,
		_bridge_transfer_id: BridgeTransferId<&'static str>,
	) -> BridgeContractResult<()> {
		Ok(())
	}

	async fn get_bridge_transfer_details(
		&self,
		_bridge_transfer_id: &'static str,
	) -> BridgeContractResult<Option<BridgeTransferDetails<&'static str, &'static str>>> {
		Ok(None)
	}
}

#[tokio::test]
async fn test_bridge_transfer_initiated() {
	let initiator_monitoring = MockInitiatorMonitoring {
		events: vec![BridgeContractInitiatorEvent::BridgeTransferInitiated(
			BridgeTransferDetails {
				bridge_transfer_id: BridgeTransferId("transfer_id"),
				initiator_address: "initiator",
				recipient_address: "recipient",
				hash_lock: "hash_lock",
				time_lock: 100,
				amount: 1000,
			},
		)],
	};

	let counterparty_monitoring = MockCounterpartyMonitoring;
	let initiator_contract = MockInitiatorContract;
	let counterparty_contract = MockCounterpartyContract;

	let mut blockchain_service = MockBlockchainService {
		initiator_contract,
		initiator_monitoring,
		counterparty_contract,
		counterparty_monitoring,
	};

	let mut cx = Context::from_waker(futures::task::noop_waker_ref());
	let event = blockchain_service.poll_next_unpin(&mut cx);

	assert_eq!(
		event,
		Poll::Ready(Some(BlockchainEvent::InitiatorEvent(
			BridgeContractInitiatorEvent::BridgeTransferInitiated(BridgeTransferDetails {
				bridge_transfer_id: BridgeTransferId("transfer_id"),
				initiator_address: "initiator",
				recipient_address: "recipient",
				hash_lock: "hash_lock",
				time_lock: 100,
				amount: 1000,
			})
		)))
	);
}
