// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use super::address::Address;
use super::coin_metadata::CoinMetadata;
use super::cursor::Page;
use super::dynamic_field::DynamicField;
use super::dynamic_field::DynamicFieldName;
use super::move_package::MovePackage;
use super::object::ObjectVersionKey;
use super::stake::StakedSui;
use super::suins_registration::SuinsRegistration;
use crate::data::Db;
use crate::types::balance::{self, Balance};
use crate::types::coin::Coin;
use crate::types::move_object::MoveObject;
use crate::types::object::{self, Object, ObjectFilter};
use crate::types::sui_address::SuiAddress;
use crate::types::type_filter::ExactTypeFilter;

use async_graphql::connection::Connection;
use async_graphql::*;
use sui_json_rpc::name_service::NameServiceConfig;
use sui_types::dynamic_field::DynamicFieldType;
use sui_types::gas_coin::GAS;

#[derive(Clone, Debug)]
pub(crate) struct Owner {
    pub address: SuiAddress,
}

/// Type to implement GraphQL fields that are shared by all Owners.
pub(crate) struct OwnerImpl(pub SuiAddress);

/// Interface implemented by GraphQL types representing entities that can own objects. Object owners
/// are identified by an address which can represent either the public key of an account or another
/// object. The same address can only refer to an account or an object, never both, but it is not
/// possible to know which up-front.
#[derive(Interface)]
#[graphql(
    name = "IOwner",
    field(name = "address", ty = "SuiAddress"),
    field(
        name = "objects",
        arg(name = "first", ty = "Option<u64>"),
        arg(name = "after", ty = "Option<object::Cursor>"),
        arg(name = "last", ty = "Option<u64>"),
        arg(name = "before", ty = "Option<object::Cursor>"),
        arg(name = "filter", ty = "Option<ObjectFilter>"),
        ty = "Connection<String, MoveObject>",
        desc = "Objects owned by this object or address, optionally `filter`-ed."
    ),
    field(
        name = "balance",
        arg(name = "type", ty = "Option<ExactTypeFilter>"),
        ty = "Option<Balance>",
        desc = "Total balance of all coins with marker type owned by this object or address. If \
                type is not supplied, it defaults to `0x2::sui::SUI`."
    ),
    field(
        name = "balances",
        arg(name = "first", ty = "Option<u64>"),
        arg(name = "after", ty = "Option<balance::Cursor>"),
        arg(name = "last", ty = "Option<u64>"),
        arg(name = "before", ty = "Option<balance::Cursor>"),
        ty = "Connection<String, Balance>",
        desc = "The balances of all coin types owned by this object or address."
    ),
    field(
        name = "coins",
        arg(name = "first", ty = "Option<u64>"),
        arg(name = "after", ty = "Option<object::Cursor>"),
        arg(name = "last", ty = "Option<u64>"),
        arg(name = "before", ty = "Option<object::Cursor>"),
        arg(name = "type", ty = "Option<ExactTypeFilter>"),
        ty = "Connection<String, Coin>",
        desc = "The coin objects for this object or address.\n\n\
                `type` is a filter on the coin's type parameter, defaulting to `0x2::sui::SUI`."
    ),
    field(
        name = "staked_suis",
        arg(name = "first", ty = "Option<u64>"),
        arg(name = "after", ty = "Option<object::Cursor>"),
        arg(name = "last", ty = "Option<u64>"),
        arg(name = "before", ty = "Option<object::Cursor>"),
        ty = "Connection<String, StakedSui>",
        desc = "The `0x3::staking_pool::StakedSui` objects owned by this object or address."
    ),
    field(
        name = "default_suins_name",
        ty = "Option<String>",
        desc = "The domain explicitly configured as the default domain pointing to this object or \
                address."
    ),
    field(
        name = "suins_registrations",
        arg(name = "first", ty = "Option<u64>"),
        arg(name = "after", ty = "Option<object::Cursor>"),
        arg(name = "last", ty = "Option<u64>"),
        arg(name = "before", ty = "Option<object::Cursor>"),
        ty = "Connection<String, SuinsRegistration>",
        desc = "The SuinsRegistration NFTs owned by this object or address. These grant the owner \
                the capability to manage the associated domain."
    )
)]
pub(crate) enum IOwner {
    Owner(Owner),
    Address(Address),
    Object(Object),
    MovePackage(MovePackage),
    MoveObject(MoveObject),
    Coin(Coin),
    CoinMetadata(CoinMetadata),
    StakedSui(StakedSui),
    SuinsRegistration(SuinsRegistration),
}

/// An Owner is an entity that can own an object. Each Owner is identified by a SuiAddress which
/// represents either an Address (corresponding to a public key of an account) or an Object, but
/// never both (it is not known up-front whether a given Owner is an Address or an Object).
#[Object]
impl Owner {
    pub(crate) async fn address(&self) -> SuiAddress {
        OwnerImpl(self.address).address().await
    }

    /// Objects owned by this object or address, optionally `filter`-ed.
    pub(crate) async fn objects(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
        filter: Option<ObjectFilter>,
    ) -> Result<Connection<String, MoveObject>> {
        OwnerImpl(self.address)
            .objects(ctx, first, after, last, before, filter)
            .await
    }

    /// Total balance of all coins with marker type owned by this object or address. If type is not
    /// supplied, it defaults to `0x2::sui::SUI`.
    pub(crate) async fn balance(
        &self,
        ctx: &Context<'_>,
        type_: Option<ExactTypeFilter>,
    ) -> Result<Option<Balance>> {
        OwnerImpl(self.address).balance(ctx, type_).await
    }

    /// The balances of all coin types owned by this object or address.
    pub(crate) async fn balances(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<balance::Cursor>,
        last: Option<u64>,
        before: Option<balance::Cursor>,
    ) -> Result<Connection<String, Balance>> {
        OwnerImpl(self.address)
            .balances(ctx, first, after, last, before)
            .await
    }

    /// The coin objects for this object or address.
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
        OwnerImpl(self.address)
            .coins(ctx, first, after, last, before, type_)
            .await
    }

    /// The `0x3::staking_pool::StakedSui` objects owned by this object or address.
    pub(crate) async fn staked_suis(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
    ) -> Result<Connection<String, StakedSui>> {
        OwnerImpl(self.address)
            .staked_suis(ctx, first, after, last, before)
            .await
    }

    /// The domain explicitly configured as the default domain pointing to this object or address.
    pub(crate) async fn default_suins_name(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        OwnerImpl(self.address).default_suins_name(ctx).await
    }

    /// The SuinsRegistration NFTs owned by this object or address. These grant the owner the
    /// capability to manage the associated domain.
    pub(crate) async fn suins_registrations(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
    ) -> Result<Connection<String, SuinsRegistration>> {
        OwnerImpl(self.address)
            .suins_registrations(ctx, first, after, last, before)
            .await
    }

    async fn as_address(&self) -> Option<Address> {
        // For now only addresses can be owners
        Some(Address {
            address: self.address,
        })
    }

    async fn as_object(&self, ctx: &Context<'_>) -> Result<Option<Object>> {
        // TODO: Make consistent
        Object::query(ctx.data_unchecked(), self.address, ObjectVersionKey::Latest)
            .await
            .extend()
    }

    /// Access a dynamic field on an object using its name. Names are arbitrary Move values whose
    /// type have `copy`, `drop`, and `store`, and are specified using their type, and their BCS
    /// contents, Base64 encoded.
    ///
    /// This field exists as a convenience when accessing a dynamic field on a wrapped object.
    async fn dynamic_field(
        &self,
        ctx: &Context<'_>,
        name: DynamicFieldName,
    ) -> Result<Option<DynamicField>> {
        OwnerImpl(self.address).dynamic_field(ctx, name).await
    }

    /// Access a dynamic object field on an object using its name. Names are arbitrary Move values
    /// whose type have `copy`, `drop`, and `store`, and are specified using their type, and their
    /// BCS contents, Base64 encoded. The value of a dynamic object field can also be accessed
    /// off-chain directly via its address (e.g. using `Query.object`).
    ///
    /// This field exists as a convenience when accessing a dynamic field on a wrapped object.
    async fn dynamic_object_field(
        &self,
        ctx: &Context<'_>,
        name: DynamicFieldName,
    ) -> Result<Option<DynamicField>> {
        OwnerImpl(self.address)
            .dynamic_object_field(ctx, name)
            .await
    }

    /// The dynamic fields and dynamic object fields on an object.
    ///
    /// This field exists as a convenience when accessing a dynamic field on a wrapped object.
    async fn dynamic_fields(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
    ) -> Result<Connection<String, DynamicField>> {
        OwnerImpl(self.address)
            .dynamic_fields(ctx, first, after, last, before)
            .await
    }
}

impl OwnerImpl {
    pub(crate) async fn address(&self) -> SuiAddress {
        self.0
    }

    pub(crate) async fn objects(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
        filter: Option<ObjectFilter>,
    ) -> Result<Connection<String, MoveObject>> {
        let page = Page::from_params(ctx.data_unchecked(), first, after, last, before)?;

        let Some(filter) = filter.unwrap_or_default().intersect(ObjectFilter {
            owner: Some(self.0),
            ..Default::default()
        }) else {
            return Ok(Connection::new(false, false));
        };

        MoveObject::paginate(ctx.data_unchecked(), page, filter)
            .await
            .extend()
    }

    pub(crate) async fn balance(
        &self,
        ctx: &Context<'_>,
        type_: Option<ExactTypeFilter>,
    ) -> Result<Option<Balance>> {
        let coin = type_.map_or_else(GAS::type_tag, |t| t.0);
        Balance::query(ctx.data_unchecked(), self.0, coin)
            .await
            .extend()
    }

    pub(crate) async fn balances(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<balance::Cursor>,
        last: Option<u64>,
        before: Option<balance::Cursor>,
    ) -> Result<Connection<String, Balance>> {
        let page = Page::from_params(ctx.data_unchecked(), first, after, last, before)?;
        Balance::paginate(ctx.data_unchecked(), page, self.0)
            .await
            .extend()
    }

    pub(crate) async fn coins(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
        type_: Option<ExactTypeFilter>,
    ) -> Result<Connection<String, Coin>> {
        let page = Page::from_params(ctx.data_unchecked(), first, after, last, before)?;
        let coin = type_.map_or_else(GAS::type_tag, |t| t.0);
        Coin::paginate(ctx.data_unchecked(), page, coin, Some(self.0))
            .await
            .extend()
    }

    pub(crate) async fn staked_suis(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
    ) -> Result<Connection<String, StakedSui>> {
        let page = Page::from_params(ctx.data_unchecked(), first, after, last, before)?;
        StakedSui::paginate(ctx.data_unchecked(), page, self.0)
            .await
            .extend()
    }

    pub(crate) async fn default_suins_name(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        Ok(SuinsRegistration::reverse_resolve_to_name(
            ctx.data_unchecked::<Db>(),
            ctx.data_unchecked::<NameServiceConfig>(),
            self.0,
        )
        .await
        .extend()?
        .map(|d| d.to_string()))
    }

    pub(crate) async fn suins_registrations(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
    ) -> Result<Connection<String, SuinsRegistration>> {
        let page = Page::from_params(ctx.data_unchecked(), first, after, last, before)?;
        SuinsRegistration::paginate(
            ctx.data_unchecked::<Db>(),
            ctx.data_unchecked::<NameServiceConfig>(),
            page,
            self.0,
        )
        .await
        .extend()
    }

    // Dynamic field related functions are part of the `IMoveObject` interface, but are provided
    // here to implement convenience functions on `Owner` and `Object` to access dynamic fields.

    pub(crate) async fn dynamic_field(
        &self,
        ctx: &Context<'_>,
        name: DynamicFieldName,
    ) -> Result<Option<DynamicField>> {
        use DynamicFieldType as T;
        DynamicField::query(ctx.data_unchecked(), self.0, name, T::DynamicField)
            .await
            .extend()
    }

    pub(crate) async fn dynamic_object_field(
        &self,
        ctx: &Context<'_>,
        name: DynamicFieldName,
    ) -> Result<Option<DynamicField>> {
        use DynamicFieldType as T;
        DynamicField::query(ctx.data_unchecked(), self.0, name, T::DynamicObject)
            .await
            .extend()
    }

    pub(crate) async fn dynamic_fields(
        &self,
        ctx: &Context<'_>,
        first: Option<u64>,
        after: Option<object::Cursor>,
        last: Option<u64>,
        before: Option<object::Cursor>,
    ) -> Result<Connection<String, DynamicField>> {
        let page = Page::from_params(ctx.data_unchecked(), first, after, last, before)?;
        DynamicField::paginate(ctx.data_unchecked(), page, self.0)
            .await
            .extend()
    }
}
