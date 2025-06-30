#[derive(Debug, Clone, Copy, postgres_types::ToSql, postgres_types::FromSql)]
#[postgres(name = "status")]
pub enum Status {
    #[postgres(name = "op!en")]
    Open,
    #[postgres(name = "clo@sed")]
    Closed,
}
pub struct ListCitiesRow {
    pub city_slug: String,
    pub city_name: String,
}
impl ListCitiesRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            city_slug: row.try_get(0)?,
            city_name: row.try_get(1)?,
        })
    }
}
pub struct ListCities;
impl ListCities {
    pub const QUERY: &'static str = r"SELECT slug, name
FROM city
ORDER BY name";
    pub async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<ListCitiesRow>, tokio_postgres::Error> {
        let rows = client.query(Self::QUERY, &[]).await?;
        rows.into_iter()
            .map(|r| ListCitiesRow::from_row(&r))
            .collect()
    }
}
pub struct GetCityRow {
    pub city_slug: String,
    pub city_name: String,
}
impl GetCityRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            city_slug: row.try_get(0)?,
            city_name: row.try_get(1)?,
        })
    }
}
pub struct GetCity<'a> {
    city_slug: &'a str,
}
impl<'a> GetCity<'a> {
    pub const QUERY: &'static str = r"SELECT slug, name
FROM city
WHERE slug = $1";
    pub async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<GetCityRow, tokio_postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.city_slug]).await?;
        GetCityRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<GetCityRow>, tokio_postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.city_slug]).await?;
        match row {
            Some(row) => Ok(Some(GetCityRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> GetCity<'a> {
    pub const fn builder() -> GetCityBuilder<'a, ((),)> {
        GetCityBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct GetCityBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetCityBuilder<'a, ((),)> {
    pub fn city_slug(self, city_slug: &'a str) -> GetCityBuilder<'a, (&'a str,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        GetCityBuilder {
            fields: (city_slug,),
            _phantom,
        }
    }
}
impl<'a> GetCityBuilder<'a, (&'a str,)> {
    pub const fn build(self) -> GetCity<'a> {
        let (city_slug,) = self.fields;
        GetCity { city_slug }
    }
}
pub struct CreateCityRow {
    pub city_slug: String,
    pub city_name: String,
}
impl CreateCityRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            city_slug: row.try_get(0)?,
            city_name: row.try_get(1)?,
        })
    }
}
pub struct CreateCity<'a> {
    city_name: &'a str,
    city_slug: &'a str,
}
impl<'a> CreateCity<'a> {
    pub const QUERY: &'static str = r"INSERT INTO city (
    name,
    slug
) VALUES (
    $1,
    $2
) RETURNING slug, name";
    pub async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<CreateCityRow, tokio_postgres::Error> {
        let row = client
            .query_one(Self::QUERY, &[&self.city_name, &self.city_slug])
            .await?;
        CreateCityRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<CreateCityRow>, tokio_postgres::Error> {
        let row = client
            .query_opt(Self::QUERY, &[&self.city_name, &self.city_slug])
            .await?;
        match row {
            Some(row) => Ok(Some(CreateCityRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> CreateCity<'a> {
    pub const fn builder() -> CreateCityBuilder<'a, ((), ())> {
        CreateCityBuilder {
            fields: ((), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct CreateCityBuilder<'a, Fields = ((), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, CitySlug> CreateCityBuilder<'a, ((), CitySlug)> {
    pub fn city_name(self, city_name: &'a str) -> CreateCityBuilder<'a, (&'a str, CitySlug)> {
        let ((), city_slug) = self.fields;
        let _phantom = self._phantom;
        CreateCityBuilder {
            fields: (city_name, city_slug),
            _phantom,
        }
    }
}
impl<'a, CityName> CreateCityBuilder<'a, (CityName, ())> {
    pub fn city_slug(self, city_slug: &'a str) -> CreateCityBuilder<'a, (CityName, &'a str)> {
        let (city_name, ()) = self.fields;
        let _phantom = self._phantom;
        CreateCityBuilder {
            fields: (city_name, city_slug),
            _phantom,
        }
    }
}
impl<'a> CreateCityBuilder<'a, (&'a str, &'a str)> {
    pub const fn build(self) -> CreateCity<'a> {
        let (city_name, city_slug) = self.fields;
        CreateCity {
            city_name,
            city_slug,
        }
    }
}
pub struct UpdateCityNameRow {}
impl UpdateCityNameRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {})
    }
}
pub struct UpdateCityName<'a> {
    city_slug: &'a str,
    city_name: &'a str,
}
impl<'a> UpdateCityName<'a> {
    pub const QUERY: &'static str = r"UPDATE city
SET name = $2
WHERE slug = $1";
    pub async fn execute(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(Self::QUERY, &[&self.city_slug, &self.city_name])
            .await
    }
}
impl<'a> UpdateCityName<'a> {
    pub const fn builder() -> UpdateCityNameBuilder<'a, ((), ())> {
        UpdateCityNameBuilder {
            fields: ((), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct UpdateCityNameBuilder<'a, Fields = ((), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, CityName> UpdateCityNameBuilder<'a, ((), CityName)> {
    pub fn city_slug(self, city_slug: &'a str) -> UpdateCityNameBuilder<'a, (&'a str, CityName)> {
        let ((), city_name) = self.fields;
        let _phantom = self._phantom;
        UpdateCityNameBuilder {
            fields: (city_slug, city_name),
            _phantom,
        }
    }
}
impl<'a, CitySlug> UpdateCityNameBuilder<'a, (CitySlug, ())> {
    pub fn city_name(self, city_name: &'a str) -> UpdateCityNameBuilder<'a, (CitySlug, &'a str)> {
        let (city_slug, ()) = self.fields;
        let _phantom = self._phantom;
        UpdateCityNameBuilder {
            fields: (city_slug, city_name),
            _phantom,
        }
    }
}
impl<'a> UpdateCityNameBuilder<'a, (&'a str, &'a str)> {
    pub const fn build(self) -> UpdateCityName<'a> {
        let (city_slug, city_name) = self.fields;
        UpdateCityName {
            city_slug,
            city_name,
        }
    }
}
pub struct ListVenuesRow {
    pub venue_id: i32,
    pub venue_status: Status,
    pub venue_statuses: Option<Vec<Status>>,
    pub venue_slug: String,
    pub venue_name: String,
    pub venue_city: String,
    pub venue_spotify_playlist: String,
    pub venue_songkick_id: Option<String>,
    pub venue_tags: Option<Vec<String>>,
    pub venue_created_at: std::time::SystemTime,
}
impl ListVenuesRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            venue_id: row.try_get(0)?,
            venue_status: row.try_get(1)?,
            venue_statuses: row.try_get(2)?,
            venue_slug: row.try_get(3)?,
            venue_name: row.try_get(4)?,
            venue_city: row.try_get(5)?,
            venue_spotify_playlist: row.try_get(6)?,
            venue_songkick_id: row.try_get(7)?,
            venue_tags: row.try_get(8)?,
            venue_created_at: row.try_get(9)?,
        })
    }
}
pub struct ListVenues<'a> {
    venue_city: &'a str,
}
impl<'a> ListVenues<'a> {
    pub const QUERY: &'static str = r"SELECT id, status, statuses, slug, name, city, spotify_playlist, songkick_id, tags, created_at
FROM venue
WHERE city = $1
ORDER BY name";
    pub async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<ListVenuesRow>, tokio_postgres::Error> {
        let rows = client.query(Self::QUERY, &[&self.venue_city]).await?;
        rows.into_iter()
            .map(|r| ListVenuesRow::from_row(&r))
            .collect()
    }
}
impl<'a> ListVenues<'a> {
    pub const fn builder() -> ListVenuesBuilder<'a, ((),)> {
        ListVenuesBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct ListVenuesBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> ListVenuesBuilder<'a, ((),)> {
    pub fn venue_city(self, venue_city: &'a str) -> ListVenuesBuilder<'a, (&'a str,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        ListVenuesBuilder {
            fields: (venue_city,),
            _phantom,
        }
    }
}
impl<'a> ListVenuesBuilder<'a, (&'a str,)> {
    pub const fn build(self) -> ListVenues<'a> {
        let (venue_city,) = self.fields;
        ListVenues { venue_city }
    }
}
pub struct DeleteVenueRow {}
impl DeleteVenueRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {})
    }
}
pub struct DeleteVenue<'a> {
    venue_slug: &'a str,
}
impl<'a> DeleteVenue<'a> {
    pub const QUERY: &'static str = r"DELETE FROM venue
WHERE slug = $1 AND slug = $1";
    pub async fn execute(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(Self::QUERY, &[&self.venue_slug]).await
    }
}
impl<'a> DeleteVenue<'a> {
    pub const fn builder() -> DeleteVenueBuilder<'a, ((),)> {
        DeleteVenueBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct DeleteVenueBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> DeleteVenueBuilder<'a, ((),)> {
    pub fn venue_slug(self, venue_slug: &'a str) -> DeleteVenueBuilder<'a, (&'a str,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        DeleteVenueBuilder {
            fields: (venue_slug,),
            _phantom,
        }
    }
}
impl<'a> DeleteVenueBuilder<'a, (&'a str,)> {
    pub const fn build(self) -> DeleteVenue<'a> {
        let (venue_slug,) = self.fields;
        DeleteVenue { venue_slug }
    }
}
pub struct GetVenueRow {
    pub venue_id: i32,
    pub venue_status: Status,
    pub venue_statuses: Option<Vec<Status>>,
    pub venue_slug: String,
    pub venue_name: String,
    pub venue_city: String,
    pub venue_spotify_playlist: String,
    pub venue_songkick_id: Option<String>,
    pub venue_tags: Option<Vec<String>>,
    pub venue_created_at: std::time::SystemTime,
}
impl GetVenueRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            venue_id: row.try_get(0)?,
            venue_status: row.try_get(1)?,
            venue_statuses: row.try_get(2)?,
            venue_slug: row.try_get(3)?,
            venue_name: row.try_get(4)?,
            venue_city: row.try_get(5)?,
            venue_spotify_playlist: row.try_get(6)?,
            venue_songkick_id: row.try_get(7)?,
            venue_tags: row.try_get(8)?,
            venue_created_at: row.try_get(9)?,
        })
    }
}
pub struct GetVenue<'a> {
    venue_slug: &'a str,
    venue_city: &'a str,
}
impl<'a> GetVenue<'a> {
    pub const QUERY: &'static str = r"SELECT id, status, statuses, slug, name, city, spotify_playlist, songkick_id, tags, created_at
FROM venue
WHERE slug = $1 AND city = $2";
    pub async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<GetVenueRow, tokio_postgres::Error> {
        let row = client
            .query_one(Self::QUERY, &[&self.venue_slug, &self.venue_city])
            .await?;
        GetVenueRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<GetVenueRow>, tokio_postgres::Error> {
        let row = client
            .query_opt(Self::QUERY, &[&self.venue_slug, &self.venue_city])
            .await?;
        match row {
            Some(row) => Ok(Some(GetVenueRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> GetVenue<'a> {
    pub const fn builder() -> GetVenueBuilder<'a, ((), ())> {
        GetVenueBuilder {
            fields: ((), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct GetVenueBuilder<'a, Fields = ((), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, VenueCity> GetVenueBuilder<'a, ((), VenueCity)> {
    pub fn venue_slug(self, venue_slug: &'a str) -> GetVenueBuilder<'a, (&'a str, VenueCity)> {
        let ((), venue_city) = self.fields;
        let _phantom = self._phantom;
        GetVenueBuilder {
            fields: (venue_slug, venue_city),
            _phantom,
        }
    }
}
impl<'a, VenueSlug> GetVenueBuilder<'a, (VenueSlug, ())> {
    pub fn venue_city(self, venue_city: &'a str) -> GetVenueBuilder<'a, (VenueSlug, &'a str)> {
        let (venue_slug, ()) = self.fields;
        let _phantom = self._phantom;
        GetVenueBuilder {
            fields: (venue_slug, venue_city),
            _phantom,
        }
    }
}
impl<'a> GetVenueBuilder<'a, (&'a str, &'a str)> {
    pub const fn build(self) -> GetVenue<'a> {
        let (venue_slug, venue_city) = self.fields;
        GetVenue {
            venue_slug,
            venue_city,
        }
    }
}
pub struct CreateVenueRow {
    pub venue_id: i32,
}
impl CreateVenueRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            venue_id: row.try_get(0)?,
        })
    }
}
pub struct CreateVenue<'a> {
    venue_slug: &'a str,
    venue_name: &'a str,
    venue_city: &'a str,
    venue_spotify_playlist: &'a str,
    venue_status: Status,
    venue_statuses: Option<&'a [Status]>,
    venue_tags: Option<&'a [String]>,
}
impl<'a> CreateVenue<'a> {
    pub const QUERY: &'static str = r"INSERT INTO venue (
    slug,
    name,
    city,
    created_at,
    spotify_playlist,
    status,
    statuses,
    tags
) VALUES (
    $1,
    $2,
    $3,
    NOW(),
    $4,
    $5,
    $6,
    $7
) RETURNING id";
    pub async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<CreateVenueRow, tokio_postgres::Error> {
        let row = client
            .query_one(
                Self::QUERY,
                &[
                    &self.venue_slug,
                    &self.venue_name,
                    &self.venue_city,
                    &self.venue_spotify_playlist,
                    &self.venue_status,
                    &self.venue_statuses,
                    &self.venue_tags,
                ],
            )
            .await?;
        CreateVenueRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<CreateVenueRow>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                Self::QUERY,
                &[
                    &self.venue_slug,
                    &self.venue_name,
                    &self.venue_city,
                    &self.venue_spotify_playlist,
                    &self.venue_status,
                    &self.venue_statuses,
                    &self.venue_tags,
                ],
            )
            .await?;
        match row {
            Some(row) => Ok(Some(CreateVenueRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> CreateVenue<'a> {
    pub const fn builder() -> CreateVenueBuilder<'a, ((), (), (), (), (), (), ())> {
        CreateVenueBuilder {
            fields: ((), (), (), (), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct CreateVenueBuilder<'a, Fields = ((), (), (), (), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, VenueName, VenueCity, VenueSpotifyPlaylist, VenueStatus, VenueStatuses, VenueTags>
    CreateVenueBuilder<
        'a,
        (
            (),
            VenueName,
            VenueCity,
            VenueSpotifyPlaylist,
            VenueStatus,
            VenueStatuses,
            VenueTags,
        ),
    >
{
    pub fn venue_slug(
        self,
        venue_slug: &'a str,
    ) -> CreateVenueBuilder<
        'a,
        (
            &'a str,
            VenueName,
            VenueCity,
            VenueSpotifyPlaylist,
            VenueStatus,
            VenueStatuses,
            VenueTags,
        ),
    > {
        let (
            (),
            venue_name,
            venue_city,
            venue_spotify_playlist,
            venue_status,
            venue_statuses,
            venue_tags,
        ) = self.fields;
        let _phantom = self._phantom;
        CreateVenueBuilder {
            fields: (
                venue_slug,
                venue_name,
                venue_city,
                venue_spotify_playlist,
                venue_status,
                venue_statuses,
                venue_tags,
            ),
            _phantom,
        }
    }
}
impl<'a, VenueSlug, VenueCity, VenueSpotifyPlaylist, VenueStatus, VenueStatuses, VenueTags>
    CreateVenueBuilder<
        'a,
        (
            VenueSlug,
            (),
            VenueCity,
            VenueSpotifyPlaylist,
            VenueStatus,
            VenueStatuses,
            VenueTags,
        ),
    >
{
    pub fn venue_name(
        self,
        venue_name: &'a str,
    ) -> CreateVenueBuilder<
        'a,
        (
            VenueSlug,
            &'a str,
            VenueCity,
            VenueSpotifyPlaylist,
            VenueStatus,
            VenueStatuses,
            VenueTags,
        ),
    > {
        let (
            venue_slug,
            (),
            venue_city,
            venue_spotify_playlist,
            venue_status,
            venue_statuses,
            venue_tags,
        ) = self.fields;
        let _phantom = self._phantom;
        CreateVenueBuilder {
            fields: (
                venue_slug,
                venue_name,
                venue_city,
                venue_spotify_playlist,
                venue_status,
                venue_statuses,
                venue_tags,
            ),
            _phantom,
        }
    }
}
impl<'a, VenueSlug, VenueName, VenueSpotifyPlaylist, VenueStatus, VenueStatuses, VenueTags>
    CreateVenueBuilder<
        'a,
        (
            VenueSlug,
            VenueName,
            (),
            VenueSpotifyPlaylist,
            VenueStatus,
            VenueStatuses,
            VenueTags,
        ),
    >
{
    pub fn venue_city(
        self,
        venue_city: &'a str,
    ) -> CreateVenueBuilder<
        'a,
        (
            VenueSlug,
            VenueName,
            &'a str,
            VenueSpotifyPlaylist,
            VenueStatus,
            VenueStatuses,
            VenueTags,
        ),
    > {
        let (
            venue_slug,
            venue_name,
            (),
            venue_spotify_playlist,
            venue_status,
            venue_statuses,
            venue_tags,
        ) = self.fields;
        let _phantom = self._phantom;
        CreateVenueBuilder {
            fields: (
                venue_slug,
                venue_name,
                venue_city,
                venue_spotify_playlist,
                venue_status,
                venue_statuses,
                venue_tags,
            ),
            _phantom,
        }
    }
}
impl<'a, VenueSlug, VenueName, VenueCity, VenueStatus, VenueStatuses, VenueTags>
    CreateVenueBuilder<
        'a,
        (
            VenueSlug,
            VenueName,
            VenueCity,
            (),
            VenueStatus,
            VenueStatuses,
            VenueTags,
        ),
    >
{
    pub fn venue_spotify_playlist(
        self,
        venue_spotify_playlist: &'a str,
    ) -> CreateVenueBuilder<
        'a,
        (
            VenueSlug,
            VenueName,
            VenueCity,
            &'a str,
            VenueStatus,
            VenueStatuses,
            VenueTags,
        ),
    > {
        let (venue_slug, venue_name, venue_city, (), venue_status, venue_statuses, venue_tags) =
            self.fields;
        let _phantom = self._phantom;
        CreateVenueBuilder {
            fields: (
                venue_slug,
                venue_name,
                venue_city,
                venue_spotify_playlist,
                venue_status,
                venue_statuses,
                venue_tags,
            ),
            _phantom,
        }
    }
}
impl<'a, VenueSlug, VenueName, VenueCity, VenueSpotifyPlaylist, VenueStatuses, VenueTags>
    CreateVenueBuilder<
        'a,
        (
            VenueSlug,
            VenueName,
            VenueCity,
            VenueSpotifyPlaylist,
            (),
            VenueStatuses,
            VenueTags,
        ),
    >
{
    pub fn venue_status(
        self,
        venue_status: Status,
    ) -> CreateVenueBuilder<
        'a,
        (
            VenueSlug,
            VenueName,
            VenueCity,
            VenueSpotifyPlaylist,
            Status,
            VenueStatuses,
            VenueTags,
        ),
    > {
        let (
            venue_slug,
            venue_name,
            venue_city,
            venue_spotify_playlist,
            (),
            venue_statuses,
            venue_tags,
        ) = self.fields;
        let _phantom = self._phantom;
        CreateVenueBuilder {
            fields: (
                venue_slug,
                venue_name,
                venue_city,
                venue_spotify_playlist,
                venue_status,
                venue_statuses,
                venue_tags,
            ),
            _phantom,
        }
    }
}
impl<'a, VenueSlug, VenueName, VenueCity, VenueSpotifyPlaylist, VenueStatus, VenueTags>
    CreateVenueBuilder<
        'a,
        (
            VenueSlug,
            VenueName,
            VenueCity,
            VenueSpotifyPlaylist,
            VenueStatus,
            (),
            VenueTags,
        ),
    >
{
    pub fn venue_statuses(
        self,
        venue_statuses: Option<&'a [Status]>,
    ) -> CreateVenueBuilder<
        'a,
        (
            VenueSlug,
            VenueName,
            VenueCity,
            VenueSpotifyPlaylist,
            VenueStatus,
            Option<&'a [Status]>,
            VenueTags,
        ),
    > {
        let (
            venue_slug,
            venue_name,
            venue_city,
            venue_spotify_playlist,
            venue_status,
            (),
            venue_tags,
        ) = self.fields;
        let _phantom = self._phantom;
        CreateVenueBuilder {
            fields: (
                venue_slug,
                venue_name,
                venue_city,
                venue_spotify_playlist,
                venue_status,
                venue_statuses,
                venue_tags,
            ),
            _phantom,
        }
    }
}
impl<'a, VenueSlug, VenueName, VenueCity, VenueSpotifyPlaylist, VenueStatus, VenueStatuses>
    CreateVenueBuilder<
        'a,
        (
            VenueSlug,
            VenueName,
            VenueCity,
            VenueSpotifyPlaylist,
            VenueStatus,
            VenueStatuses,
            (),
        ),
    >
{
    pub fn venue_tags(
        self,
        venue_tags: Option<&'a [String]>,
    ) -> CreateVenueBuilder<
        'a,
        (
            VenueSlug,
            VenueName,
            VenueCity,
            VenueSpotifyPlaylist,
            VenueStatus,
            VenueStatuses,
            Option<&'a [String]>,
        ),
    > {
        let (
            venue_slug,
            venue_name,
            venue_city,
            venue_spotify_playlist,
            venue_status,
            venue_statuses,
            (),
        ) = self.fields;
        let _phantom = self._phantom;
        CreateVenueBuilder {
            fields: (
                venue_slug,
                venue_name,
                venue_city,
                venue_spotify_playlist,
                venue_status,
                venue_statuses,
                venue_tags,
            ),
            _phantom,
        }
    }
}
impl<'a>
    CreateVenueBuilder<
        'a,
        (
            &'a str,
            &'a str,
            &'a str,
            &'a str,
            Status,
            Option<&'a [Status]>,
            Option<&'a [String]>,
        ),
    >
{
    pub const fn build(self) -> CreateVenue<'a> {
        let (
            venue_slug,
            venue_name,
            venue_city,
            venue_spotify_playlist,
            venue_status,
            venue_statuses,
            venue_tags,
        ) = self.fields;
        CreateVenue {
            venue_slug,
            venue_name,
            venue_city,
            venue_spotify_playlist,
            venue_status,
            venue_statuses,
            venue_tags,
        }
    }
}
pub struct UpdateVenueNameRow {
    pub venue_id: i32,
}
impl UpdateVenueNameRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            venue_id: row.try_get(0)?,
        })
    }
}
pub struct UpdateVenueName<'a> {
    venue_slug: &'a str,
    venue_name: &'a str,
}
impl<'a> UpdateVenueName<'a> {
    pub const QUERY: &'static str = r"UPDATE venue
SET name = $2
WHERE slug = $1
RETURNING id";
    pub async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<UpdateVenueNameRow, tokio_postgres::Error> {
        let row = client
            .query_one(Self::QUERY, &[&self.venue_slug, &self.venue_name])
            .await?;
        UpdateVenueNameRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<UpdateVenueNameRow>, tokio_postgres::Error> {
        let row = client
            .query_opt(Self::QUERY, &[&self.venue_slug, &self.venue_name])
            .await?;
        match row {
            Some(row) => Ok(Some(UpdateVenueNameRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> UpdateVenueName<'a> {
    pub const fn builder() -> UpdateVenueNameBuilder<'a, ((), ())> {
        UpdateVenueNameBuilder {
            fields: ((), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct UpdateVenueNameBuilder<'a, Fields = ((), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, VenueName> UpdateVenueNameBuilder<'a, ((), VenueName)> {
    pub fn venue_slug(
        self,
        venue_slug: &'a str,
    ) -> UpdateVenueNameBuilder<'a, (&'a str, VenueName)> {
        let ((), venue_name) = self.fields;
        let _phantom = self._phantom;
        UpdateVenueNameBuilder {
            fields: (venue_slug, venue_name),
            _phantom,
        }
    }
}
impl<'a, VenueSlug> UpdateVenueNameBuilder<'a, (VenueSlug, ())> {
    pub fn venue_name(
        self,
        venue_name: &'a str,
    ) -> UpdateVenueNameBuilder<'a, (VenueSlug, &'a str)> {
        let (venue_slug, ()) = self.fields;
        let _phantom = self._phantom;
        UpdateVenueNameBuilder {
            fields: (venue_slug, venue_name),
            _phantom,
        }
    }
}
impl<'a> UpdateVenueNameBuilder<'a, (&'a str, &'a str)> {
    pub const fn build(self) -> UpdateVenueName<'a> {
        let (venue_slug, venue_name) = self.fields;
        UpdateVenueName {
            venue_slug,
            venue_name,
        }
    }
}
pub struct VenueCountByCityRow {
    pub venue_city: String,
    pub count: i64,
}
impl VenueCountByCityRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            venue_city: row.try_get(0)?,
            count: row.try_get(1)?,
        })
    }
}
pub struct VenueCountByCity;
impl VenueCountByCity {
    pub const QUERY: &'static str = r"SELECT
    city,
    count(*)
FROM venue
GROUP BY 1
ORDER BY 1";
    pub async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<VenueCountByCityRow>, tokio_postgres::Error> {
        let rows = client.query(Self::QUERY, &[]).await?;
        rows.into_iter()
            .map(|r| VenueCountByCityRow::from_row(&r))
            .collect()
    }
}
