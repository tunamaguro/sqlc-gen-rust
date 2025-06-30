#[allow(warnings)]
mod queries;

#[cfg(test)]
mod tests {
    use super::queries::*;
    use test_context::test_context;
    use test_utils::PgTokioTestContext;

    async fn migrate_db(client: &mut impl tokio_postgres::GenericClient) {
        const QUERIES: &[&str] = &[
            include_str!("../schema/0001_city.sql"),
            include_str!("../schema/0002_venue.sql"),
            include_str!("../schema/0003_add_column.sql"),
        ];

        let tx = client.transaction().await.unwrap();
        for q in QUERIES {
            tx.batch_execute(q).await.unwrap();
        }
        tx.commit().await.unwrap();
    }

    /// port from https://github.com/sqlc-dev/sqlc/blob/v1.29.0/examples/ondeck/postgresql/db_test.go
    #[test_context(PgTokioTestContext)]
    #[tokio::test]
    async fn run_on_deck_queries(ctx: &mut PgTokioTestContext) {
        let client = &mut ctx.client;
        migrate_db(client).await;

        // Create city
        let city = CreateCity::builder()
            .slug("san-francisco")
            .name("San Francisco")
            .build()
            .query_one(client)
            .await
            .unwrap();

        // Create venue
        let tags = vec!["rock".to_string(), "punk".to_string()];
        let statuses = vec![Status::Open, Status::Closed];
        let create_venue = CreateVenue::builder()
            .slug("the-fillmore")
            .name("The Fillmore")
            .city(&city.slug)
            .spotify_playlist("spotify:uri")
            .status(Status::Open)
            .statuses(Some(&statuses))
            .tags(Some(&tags))
            .build();

        let venue_id = create_venue.query_one(client).await.unwrap().id;

        // Get venue
        let venue = GetVenue::builder()
            .slug("the-fillmore")
            .city(&city.slug)
            .build()
            .query_one(client)
            .await
            .unwrap();

        assert_eq!(venue.id, venue_id);

        // Get city
        let actual_city = GetCity::builder()
            .slug(&city.slug)
            .build()
            .query_one(client)
            .await
            .unwrap();

        assert_eq!(actual_city.slug, city.slug);
        assert_eq!(actual_city.name, city.name);

        // Venue count by city
        let venue_counts = VenueCountByCity.query_many(client).await.unwrap();

        assert_eq!(venue_counts.len(), 1);
        assert_eq!(venue_counts[0].city, city.slug);
        assert_eq!(venue_counts[0].count, 1);

        // List cities
        let cities = ListCities.query_many(client).await.unwrap();

        assert_eq!(cities.len(), 1);
        assert_eq!(cities[0].slug, city.slug);
        assert_eq!(cities[0].name, city.name);

        // List venues
        let venues = ListVenues::builder()
            .city(&city.slug)
            .build()
            .query_many(client)
            .await
            .unwrap();

        assert_eq!(venues.len(), 1);
        assert_eq!(venues[0].id, venue.id);
        assert_eq!(venues[0].slug, venue.slug);
        assert_eq!(venues[0].name, venue.name);

        // Update city name
        let updated_rows = UpdateCityName::builder()
            .slug(&city.slug)
            .name("SF")
            .build()
            .execute(client)
            .await
            .unwrap();

        assert_eq!(updated_rows, 1);

        // Update venue name
        let updated_venue = UpdateVenueName::builder()
            .slug(&venue.slug)
            .name("Fillmore")
            .build()
            .query_one(client)
            .await
            .unwrap();

        assert_eq!(updated_venue.id, venue.id);

        // Delete venue
        let deleted = DeleteVenue::builder()
            .slug(&venue.slug)
            .build()
            .execute(client)
            .await
            .unwrap();

        assert_eq!(deleted, 1);
    }
}
