use diesel::{associations::HasTable, BoolExpressionMethods, ExpressionMethods, Table};
use error_stack::report;

use super::generics;
use crate::{errors, mandate::*, schema::mandate::dsl, PgPooledConn, StorageResult};

impl MandateNew {
    pub async fn insert(self, conn: &PgPooledConn) -> StorageResult<Mandate> {
        generics::generic_insert(conn, self).await
    }
}

impl Mandate {
    pub async fn find_by_merchant_id_mandate_id(
        conn: &PgPooledConn,
        merchant_id: &common_utils::id_type::MerchantId,
        mandate_id: &str,
    ) -> StorageResult<Self> {
        generics::generic_find_one::<<Self as HasTable>::Table, _, _>(
            conn,
            dsl::merchant_id
                .eq(merchant_id.to_owned())
                .and(dsl::mandate_id.eq(mandate_id.to_owned())),
        )
        .await
    }

    pub async fn find_by_merchant_id_connector_mandate_id(
        conn: &PgPooledConn,
        merchant_id: &common_utils::id_type::MerchantId,
        connector_mandate_id: &str,
    ) -> StorageResult<Self> {
        generics::generic_find_one::<<Self as HasTable>::Table, _, _>(
            conn,
            dsl::merchant_id
                .eq(merchant_id.to_owned())
                .and(dsl::connector_mandate_id.eq(connector_mandate_id.to_owned())),
        )
        .await
    }

    pub async fn find_by_merchant_id_customer_id(
        conn: &PgPooledConn,
        merchant_id: &common_utils::id_type::MerchantId,
        customer_id: &common_utils::id_type::CustomerId,
    ) -> StorageResult<Vec<Self>> {
        generics::generic_filter::<
            <Self as HasTable>::Table,
            _,
            <<Self as HasTable>::Table as Table>::PrimaryKey,
            _,
        >(
            conn,
            dsl::merchant_id
                .eq(merchant_id.to_owned())
                .and(dsl::customer_id.eq(customer_id.to_owned())),
            None,
            None,
            None,
        )
        .await
    }

    //Fix this function once V2 mandate is schema is being built
    #[cfg(feature = "v2")]
    pub async fn find_by_global_customer_id(
        conn: &PgPooledConn,
        customer_id: &common_utils::id_type::GlobalCustomerId,
    ) -> StorageResult<Vec<Self>> {
        generics::generic_filter::<
            <Self as HasTable>::Table,
            _,
            <<Self as HasTable>::Table as Table>::PrimaryKey,
            _,
        >(
            conn,
            dsl::customer_id.eq(customer_id.to_owned()),
            None,
            None,
            None,
        )
        .await
    }

    pub async fn update_by_merchant_id_mandate_id(
        conn: &PgPooledConn,
        merchant_id: &common_utils::id_type::MerchantId,
        mandate_id: &str,
        mandate: MandateUpdateInternal,
    ) -> StorageResult<Self> {
        generics::generic_update_with_results::<<Self as HasTable>::Table, _, _, _>(
            conn,
            dsl::merchant_id
                .eq(merchant_id.to_owned())
                .and(dsl::mandate_id.eq(mandate_id.to_owned())),
            mandate,
        )
        .await?
        .first()
        .cloned()
        .ok_or_else(|| {
            report!(errors::DatabaseError::NotFound)
                .attach_printable("Error while updating mandate")
        })
    }
}
