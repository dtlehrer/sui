// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::data::{Db, RawQueryBuilder};
use crate::error::Error;
use crate::filter;

use super::balance::{self, Balance};
use super::base64::Base64;
use super::big_int::BigInt;
use super::cursor::Page;
use super::display::DisplayEntry;
use super::dynamic_field::{DynamicField, DynamicFieldName};
use super::move_object::{MoveObject, MoveObjectImpl};
use super::move_value::MoveValue;
use super::object::{self, Object, ObjectFilter, ObjectImpl, ObjectOwner, ObjectStatus};
use super::owner::OwnerImpl;
use super::stake::StakedSui;
use super::sui_address::SuiAddress;
use super::suins_registration::SuinsRegistration;
use super::transaction_block::{self, TransactionBlock, TransactionBlockFilter};
use super::type_filter::ExactTypeFilter;
use async_graphql::*;

use async_graphql::connection::Connection;
use sui_indexer::types_v2::OwnerType;
use sui_types::coin::Coin as NativeCoin;
use sui_types::TypeTag;

#[derive(Clone)]
pub(crate) struct Coin {
    /// Representation of this Coin as a generic Move Object.
    pub super_: MoveObject,

    /// The deserialized representation of the Move Object's contents, as a `0x2::coin::Coin`.
    pub native: NativeCoin,
}

pub(crate) enum CoinDowncastError {
    NotACoin,
    Bcs(bcs::Error),
}

/// Some 0x2::coin::Coin Move object.
#[Object]
impl Coin {
    pub(crate) async fn address(&self) -> SuiAddress {
        OwnerImpl::from(&self.super_.super_).address().await
    }

    /// Objects owned by this object, optionally `filter`-ed.
    pub(crate) async fn objects(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
        filter: Option<ObjectFilter>,
    ) -> Result<Connection<String, MoveObject>> {
        OwnerImpl::from(&self.super_.super_)
            .objects(ctx, first, after, last, before, filter)
            .await
    }

    /// Total balance of all coins with marker type owned by this object. If type is not supplied,
    /// it defaults to `0x2::sui::SUI`.
    pub(crate) async fn balance(
        &self,
        ctx: &Context<'_>,
        type_: Option<ExactTypeFilter>,
    ) -> Result<Option<Balance>> {
        OwnerImpl::from(&self.super_.super_)
            .balance(ctx, type_)
            .await
    }

    /// The balances of all coin types owned by this object.
    pub(crate) async fn balances(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<balance::Cursor>,
        last: Option<u64>,
        before: Option<balance::Cursor>,
    ) -> Result<Connection<String, Balance>> {
        OwnerImpl::from(&self.super_.super_)
            .balances(ctx, first, after, last, before)
            .await
    }

    /// The coin objects for this object.
    ///
    ///`type` is a filter on the coin's type parameter, defaulting to `0x2::sui::SUI`.
    pub(crate) async fn coins(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
        type_: Option<ExactTypeFilter>,
    ) -> Result<Connection<String, Coin>> {
        OwnerImpl::from(&self.super_.super_)
            .coins(ctx, first, after, last, before, type_)
            .await
    }

    /// The `0x3::staking_pool::StakedSui` objects owned by this object.
    pub(crate) async fn staked_suis(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
    ) -> Result<Connection<String, StakedSui>> {
        OwnerImpl::from(&self.super_.super_)
            .staked_suis(ctx, first, after, last, before)
            .await
    }

    /// The domain explicitly configured as the default domain pointing to this object.
    pub(crate) async fn default_suins_name(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        OwnerImpl::from(&self.super_.super_)
            .default_suins_name(ctx)
            .await
    }

    /// The SuinsRegistration NFTs owned by this object. These grant the owner the capability to
    /// manage the associated domain.
    pub(crate) async fn suins_registrations(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
    ) -> Result<Connection<String, SuinsRegistration>> {
        OwnerImpl::from(&self.super_.super_)
            .suins_registrations(ctx, first, after, last, before)
            .await
    }

    pub(crate) async fn version(&self) -> u64 {
        ObjectImpl(&self.super_.super_).version().await
    }

    /// The current status of the object as read from the off-chain store. The possible states are:
    /// NOT_INDEXED, the object is loaded from serialized data, such as the contents of a genesis or
    /// system package upgrade transaction. LIVE, the version returned is the most recent for the
    /// object, and it is not deleted or wrapped at that version. HISTORICAL, the object was
    /// referenced at a specific version or checkpoint, so is fetched from historical tables and may
    /// not be the latest version of the object. WRAPPED_OR_DELETED, the object is deleted or
    /// wrapped and only partial information can be loaded."
    pub(crate) async fn status(&self) -> ObjectStatus {
        ObjectImpl(&self.super_.super_).status().await
    }

    /// 32-byte hash that identifies the object's contents, encoded as a Base58 string.
    pub(crate) async fn digest(&self) -> Option<String> {
        ObjectImpl(&self.super_.super_).digest().await
    }

    /// The owner type of this object: Immutable, Shared, Parent, Address
    pub(crate) async fn owner(&self, ctx: &Context<'_>) -> Option<ObjectOwner> {
        ObjectImpl(&self.super_.super_).owner(ctx).await
    }

    /// The transaction block that created this version of the object.
    pub(crate) async fn previous_transaction_block(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<TransactionBlock>> {
        ObjectImpl(&self.super_.super_)
            .previous_transaction_block(ctx)
            .await
    }

    /// The amount of SUI we would rebate if this object gets deleted or mutated. This number is
    /// recalculated based on the present storage gas price.
    pub(crate) async fn storage_rebate(&self) -> Option<BigInt> {
        ObjectImpl(&self.super_.super_).storage_rebate().await
    }

    /// The transaction blocks that sent objects to this object.
    pub(crate) async fn received_transaction_blocks(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<transaction_block::Cursor>,
        last: Option<u64>,
        before: Option<transaction_block::Cursor>,
        filter: Option<TransactionBlockFilter>,
    ) -> Result<Connection<String, TransactionBlock>> {
        ObjectImpl(&self.super_.super_)
            .received_transaction_blocks(ctx, first, after, last, before, filter)
            .await
    }

    /// The Base64-encoded BCS serialization of the object's content.
    pub(crate) async fn bcs(&self) -> Result<Option<Base64>> {
        ObjectImpl(&self.super_.super_).bcs().await
    }

    /// Displays the contents of the Move object in a JSON string and through GraphQL types. Also
    /// provides the flat representation of the type signature, and the BCS of the corresponding
    /// data.
    pub(crate) async fn contents(&self) -> Option<MoveValue> {
        MoveObjectImpl(&self.super_).contents().await
    }

    /// Determines whether a transaction can transfer this object, using the TransferObjects
    /// transaction command or `sui::transfer::public_transfer`, both of which require the object to
    /// have the `key` and `store` abilities.
    pub(crate) async fn has_public_transfer(&self, ctx: &Context<'_>) -> Result<bool> {
        MoveObjectImpl(&self.super_).has_public_transfer(ctx).await
    }

    /// The set of named templates defined on-chain for the type of this object, to be handled
    /// off-chain. The server substitutes data from the object into these templates to generate a
    /// display string per template.
    pub(crate) async fn display(&self, ctx: &Context<'_>) -> Result<Option<Vec<DisplayEntry>>> {
        ObjectImpl(&self.super_.super_).display(ctx).await
    }

    /// Access a dynamic field on an object using its name. Names are arbitrary Move values whose
    /// type have `copy`, `drop`, and `store`, and are specified using their type, and their BCS
    /// contents, Base64 encoded.
    ///
    /// Dynamic fields on wrapped objects can be accessed by using the same API under the Owner
    /// type.
    pub(crate) async fn dynamic_field(
        &self,
        ctx: &Context<'_>,
        name: DynamicFieldName,
    ) -> Result<Option<DynamicField>> {
        OwnerImpl::from(&self.super_.super_)
            .dynamic_field(ctx, name)
            .await
    }

    /// Access a dynamic object field on an object using its name. Names are arbitrary Move values
    /// whose type have `copy`, `drop`, and `store`, and are specified using their type, and their
    /// BCS contents, Base64 encoded. The value of a dynamic object field can also be accessed
    /// off-chain directly via its address (e.g. using `Query.object`).
    ///
    /// Dynamic fields on wrapped objects can be accessed by using the same API under the Owner
    /// type.
    pub(crate) async fn dynamic_object_field(
        &self,
        ctx: &Context<'_>,
        name: DynamicFieldName,
    ) -> Result<Option<DynamicField>> {
        OwnerImpl::from(&self.super_.super_)
            .dynamic_object_field(ctx, name)
            .await
    }

    /// The dynamic fields and dynamic object fields on an object.
    ///
    /// Dynamic fields on wrapped objects can be accessed by using the same API under the Owner
    /// type.
    pub(crate) async fn dynamic_fields(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
    ) -> Result<Connection<String, DynamicField>> {
        OwnerImpl::from(&self.super_.super_)
            .dynamic_fields(ctx, first, after, last, before)
            .await
    }

    /// Balance of this coin object.
    async fn coin_balance(&self) -> Option<BigInt> {
        Some(BigInt::from(self.native.balance.value()))
    }
}

impl Coin {
    /// Query the database for a `page` of coins. The page uses the bytes of an Object ID as the
    /// cursor, and can optionally be filtered by an owner.
    pub(crate) async fn paginate(
        db: &Db,
        page: Page<object::Cursor>,
        coin_type: TypeTag,
        owner: Option<SuiAddress>,
        checkpoint_sequence_number: Option<u64>,
    ) -> Result<Connection<String, Coin>, Error> {
        Object::paginate_subtype(
            db,
            page,
            checkpoint_sequence_number,
            move |query| Self::filter(query, coin_type.clone(), owner),
            |object| {
                let address = object.address;
                let move_object = MoveObject::try_from(&object).map_err(|_| {
                    Error::Internal(format!(
                        "Expected {address} to be a Coin, but it's not a Move Object.",
                    ))
                })?;

                Coin::try_from(&move_object).map_err(|_| {
                    Error::Internal(format!("Expected {address} to be a Coin, but it is not."))
                })
            },
        )
        .await
    }

    pub(crate) fn filter(
        mut query: RawQueryBuilder,
        coin_type: TypeTag,
        owner: Option<SuiAddress>,
    ) -> RawQueryBuilder {
        if let Some(owner) = owner {
            query = filter!(
                query,
                format!(
                    "owner_id = '\\x{}'::bytea AND owner_type = {}",
                    hex::encode(owner.into_vec()),
                    OwnerType::Address as i16
                )
            );
        }

        query = filter!(
            query,
            "coin_type IS NOT NULL AND coin_type = {}",
            coin_type.to_canonical_display(/* with_prefix */ true)
        );

        query
    }
}

impl TryFrom<&MoveObject> for Coin {
    type Error = CoinDowncastError;

    fn try_from(move_object: &MoveObject) -> Result<Self, Self::Error> {
        if !move_object.native.is_coin() {
            return Err(CoinDowncastError::NotACoin);
        }

        Ok(Self {
            super_: move_object.clone(),
            native: bcs::from_bytes(move_object.native.contents())
                .map_err(CoinDowncastError::Bcs)?,
        })
    }
}
