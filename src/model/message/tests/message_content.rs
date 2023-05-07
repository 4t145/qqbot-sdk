use super::*;
#[test]
fn test_macro() {
    assert_eq!(
        MessageContent::default().text("hello world"),
        &mut ("hello world".parse::<MessageContent>().unwrap())
    );
    assert_eq!(
        MessageContent::default().text("hello world").at(123456789).at_all(),
        &mut ("hello world<@!123456789>@everyone".parse::<MessageContent>().unwrap())
    );
    assert_eq!(
        MessageContent::default().text(">>>>>>").at(123456789).link_channel(88888).text("<<<<>>>>").at_all().text("<><>>>"),
        &mut ("&gt;&gt;&gt;&gt;&gt;&gt;<@!123456789><#88888>&lt;&lt;&lt;&gt;&gt;&gt;&gt;&gt;&gt;&lt;&lt;&lt;&gt;&gt;&gt;&gt;&gt;&gt;".parse::<MessageContent>().unwrap())
    );
    // assert_eq!(
    //     content! {
    //         at!(123456789),
    //         text!("good morning"),
    //         at!(234)
    //     },
    //     "<@!123456789>good morning<@!234>"
    // );
    // assert_eq!(
    //     content! {
    //         at!(*),
    //         text!("good morning"),
    //         at!(234)
    //     },
    //     "@everyonegood morning<@!234>"
    // );
    // assert_eq!(
    //     content! {
    //         at!(*),
    //         text!("good morning"),
    //         at!(234),
    //         channel!(123456789)
    //     },
    //     "@everyonegood morning<@!234><#123456789>"
    // );
    // assert_eq!(
    //     content! {
    //         at!(*),
    //         text!("good morning"),
    //         at!(123, 456, 789),
    //         channel!(123456789),
    //         emoji!(SystemEmoji::生气)
    //     },
    //     "@everyonegood morning<@!123><@!456><@!789><#123456789><emoji:326>"
    // );
    // assert_eq!(
    //     content! {
    //         text!(">>>>(&-**&)<<"),
    //         at!(123456789)
    //     },
    //     "&gt;&gt;&gt;&gt;(&amp;-**&amp;)&lt;&lt;<@!123456789>"
    // );


}
