use serenity::{
    client::Context,
    framework::standard::{
        macros::{check, command},
        Args, CommandResult, Reason,
    },
    model::channel::Message,
};

#[command] // gonna remove this soon but not now
#[checks(Admin)]
async fn testadmin(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "You have admin perms").await?;
    Ok(())
}

#[check]
#[name = "Admin"]
async fn admin_check(ctx: &Context, msg: &Message, _args: &mut Args) -> Result<(), Reason> {
    let member = msg.member(ctx).await;
    if let Ok(member) = member {
        let perms = member.permissions(ctx);
        if let Ok(perms) = perms {
            if perms.administrator() {
                return Ok(());
            } else {
                return Err(Reason::Unknown);
            }
        }
    }
    Err(Reason::Unknown)
}
