mod model;
use model::*;

use admin_macro::ModelAdmin;
use seaorm_admin::Admin;

#[derive(ModelAdmin, Default)]
#[model_admin(module = cake)]
struct CakeAdmin;

#[test]
fn test_default() {
    let mut admin = Admin::new(connection, "/admin");
    admin.add_model(CakeAdmin);
}
