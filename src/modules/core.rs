use std::collections::HashSet;


use serenity::{
    prelude::*,
    framework::standard::{
        help_commands,
        macros::{help, command},
        HelpOptions,
        StandardFramework,
        CommandResult,
        CommandOptions,
        Args, CommandGroup,
        
    }, 
    model::{
        channel::{Message, Channel},
        id::UserId
    }
};



#[help]
#[command_not_found_text= "Command not found: {}"]
#[max_levenshtein_distance(3)]
#[indention_prefix = ">"]
#[lacking_permissions="Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn c_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
    ) -> CommandResult {
        let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
        Ok(())
}


#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    
    msg.reply(
        &ctx,
        format!("A pretty normal bot")
    ).await?;

    Ok(())
}


