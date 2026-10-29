use std::io::Cursor;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use image::{ImageFormat, Rgba, RgbaImage};
use rekall_kernel::Instant;
use rekall_core::EventBus;

use super::*;
use crate::catalog::CompanyCommand;
use crate::catalog::CompanyService;

struct Fixture {
    logos: CompanyLogoService,
    companies: CompanyService,
}

/// An in-memory catalog whose clock moves a second on every reading, so two uploads never share a version.
async fn fixture() -> Fixture {
    let database = rekall_store::open_in_memory().await.unwrap();
    let mut ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let seconds = Arc::new(AtomicI64::new(1_700_000_000));
    ctx.clock = Arc::new(move || Instant::from_epoch_seconds(seconds.fetch_add(1, Ordering::SeqCst)).unwrap());
    Fixture { logos: CompanyLogoService::new(ctx.clone()), companies: CompanyService::new(ctx) }
}

impl Fixture {
    async fn company(&self, name: &str) -> Id {
        self.companies.create(CompanyCommand { name: Some(name.into()), description: None }).await.unwrap().id
    }

    async fn logo_version(&self, id: Id) -> Option<Instant> {
        self.companies.get(id).await.unwrap().logo_image_version
    }
}

fn jpeg(width: u32, height: u32) -> Vec<u8> {
    let rgb = image::DynamicImage::ImageRgba8(RgbaImage::from_pixel(width, height, Rgba([200, 90, 30, 255]))).to_rgb8();
    let mut out = Cursor::new(Vec::new());
    rgb.write_to(&mut out, ImageFormat::Jpeg).unwrap();
    out.into_inner()
}

#[tokio::test]
async fn a_stored_logo_is_rendered_as_a_png_of_the_size_asked_and_announced_on_the_company() {
    let world = fixture().await;
    let acme = world.company("Acme").await;
    assert_eq!(world.logo_version(acme).await, None, "no logo until one is uploaded");

    world.logos.store(acme, jpeg(600, 300)).await.unwrap();

    let rendered = image::load_from_memory(&world.logos.render(acme, 32).await.unwrap()).unwrap();
    assert_eq!((rendered.width(), rendered.height()), (32, 32));
    assert!(world.logo_version(acme).await.is_some());
    let listed = world.companies.list().await.unwrap();
    assert_eq!(listed[0].logo_image_version, world.logo_version(acme).await, "the list carries the same version");
}

#[tokio::test]
async fn a_new_upload_changes_the_version() {
    let world = fixture().await;
    let acme = world.company("Acme").await;
    world.logos.store(acme, jpeg(64, 64)).await.unwrap();
    let first = world.logo_version(acme).await;

    world.logos.store(acme, jpeg(64, 64)).await.unwrap();

    assert_ne!(world.logo_version(acme).await, first);
}

#[tokio::test]
async fn clearing_the_logo_returns_the_company_to_its_fallback() {
    let world = fixture().await;
    let acme = world.company("Acme").await;
    world.logos.store(acme, jpeg(64, 64)).await.unwrap();

    world.logos.clear(acme).await.unwrap();

    assert_eq!(world.logo_version(acme).await, None);
    assert!(matches!(world.logos.render(acme, 32).await, Err(RekallError::NotFound(_))));
}

#[tokio::test]
async fn a_file_that_is_not_an_image_is_refused_as_a_client_mistake_and_nothing_is_stored() {
    let world = fixture().await;
    let acme = world.company("Acme").await;

    let refused = world.logos.store(acme, b"not a picture".to_vec()).await;

    assert!(matches!(refused, Err(RekallError::IllegalArgument(_))), "{refused:?}");
    assert_eq!(world.logo_version(acme).await, None);
}

#[tokio::test]
async fn an_unknown_company_is_not_found() {
    let world = fixture().await;
    let nobody = Id::random();

    assert!(matches!(world.logos.store(nobody, jpeg(16, 16)).await, Err(RekallError::NotFound(_))));
    assert!(matches!(world.logos.clear(nobody).await, Err(RekallError::NotFound(_))));
}

#[tokio::test]
async fn deleting_a_company_deletes_its_logo() {
    let world = fixture().await;
    let acme = world.company("Acme").await;
    world.logos.store(acme, jpeg(16, 16)).await.unwrap();

    world.companies.delete(acme).await.unwrap();

    assert!(matches!(world.logos.render(acme, 16).await, Err(RekallError::NotFound(_))));
}
