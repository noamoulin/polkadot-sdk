//! # XCM Configuration

use frame::{
	deps::frame_system,
	runtime::prelude::*,
	traits::{Everything, Nothing},
};
use xcm::v3::prelude::*;
use xcm_builder::{
	AccountId32Aliases, CurrencyAdapter, DescribeAllTerminal, DescribeFamily, EnsureXcmOrigin,
	HashedDescription, IsConcrete, SignedToAccountId32,
};
use xcm_executor::XcmExecutor;

use super::{AccountId, Balances, MessageQueue, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin};

parameter_types! {
	pub RelayLocation: MultiLocation = MultiLocation::parent();
	pub ThisNetwork: NetworkId = NetworkId::Polkadot;
}

pub type LocationToAccountId = (
	HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>,
	AccountId32Aliases<ThisNetwork, AccountId>,
);

#[docify::export]
mod asset_transactor {
	use super::*;

	/// AssetTransactor for handling the relay chain token
	pub type CurrencyTransactor = CurrencyAdapter<
		// Use this Currency implementation
		Balances,
		// Use this transactor for dealing with the relay chain token
		IsConcrete<RelayLocation>,
		// How to convert an XCM MultiLocation into a local account id
		LocationToAccountId,
		// The account id type, needed because Currency is generic over it
		AccountId,
		// Not tracking teleports
		(),
	>;

	/// All asset transactors, in this case only one
	pub type AssetTransactor = CurrencyTransactor;
}

#[docify::export]
mod is_reserve {
	use super::*;

	parameter_types! {
		pub RelayTokenForRelay: (MultiAssetFilter, MultiLocation) =
		  (Wild(AllOf { id: Concrete(Parent.into()), fun: WildFungible }), Parent.into());
	}

	/// Put it all together
	pub type IsReserve = xcm_builder::Case<RelayTokenForRelay>;
}

mod weigher {
	use super::*;
	use xcm_builder::FixedWeightBounds;

	parameter_types! {
		pub const WeightPerInstruction: Weight = Weight::from_parts(1, 1);
		pub const MaxInstructions: u32 = 100;
	}

	pub type Weigher = FixedWeightBounds<WeightPerInstruction, RuntimeCall, MaxInstructions>;
}

parameter_types! {
	pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(NetworkId::Polkadot), Parachain(2222));
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = ();
	type AssetTransactor = asset_transactor::AssetTransactor;
	type OriginConverter = ();
	type IsReserve = is_reserve::IsReserve;
	type IsTeleporter = ();
	type UniversalLocation = UniversalLocation;
	// This is not safe, you should use `xcm_builder::AllowTopLevelPaidExecutionFrom<T>` in a
	// production chain
	type Barrier = xcm_builder::AllowUnpaidExecutionFrom<Everything>;
	type Weigher = weigher::Weigher;
	type Trader = ();
	type ResponseHandler = ();
	type AssetTrap = ();
	type AssetLocker = ();
	type AssetExchanger = ();
	type AssetClaims = ();
	type SubscriptionService = ();
	type PalletInstancesInfo = ();
	type FeeManager = ();
	type MaxAssetsIntoHolding = frame::traits::ConstU32<1>;
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type Aliasers = Nothing;
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, ThisNetwork>;

impl pallet_xcm::Config for Runtime {
	// We turn off sending for these tests
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, ()>;
	type XcmRouter = super::super::network::ParachainXcmRouter<MessageQueue>; // Provided by xcm-simulator
																		  // Anyone can execute XCM programs
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	// We execute any type of program
	type XcmExecuteFilter = Everything;
	// How we execute programs
	type XcmExecutor = XcmExecutor<XcmConfig>;
	// We don't allow teleports
	type XcmTeleportFilter = Nothing;
	// We allow all reserve transfers
	type XcmReserveTransferFilter = Everything;
	// Same weigher executor uses to weigh XCM programs
	type Weigher = weigher::Weigher;
	// Same universal location
	type UniversalLocation = UniversalLocation;
	// No version discovery needed
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 0;
	type AdvertisedXcmVersion = frame::traits::ConstU32<3>;
	type AdminOrigin = frame_system::EnsureRoot<AccountId>;
	// No locking
	type TrustedLockers = ();
	type MaxLockers = frame::traits::ConstU32<0>;
	type MaxRemoteLockConsumers = frame::traits::ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	// How to turn locations into accounts
	type SovereignAccountOf = LocationToAccountId;
	// A currency to pay for things and its matcher, we are using the relay token
	type Currency = Balances;
	type CurrencyMatcher = IsConcrete<RelayLocation>;
	// Pallet benchmarks, no need for this example
	type WeightInfo = pallet_xcm::TestWeightInfo;
	// Runtime types
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
}
