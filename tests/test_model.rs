use qqbot_sdk::model::MessageContent;

#[test]
fn test_macro() {
    assert_eq!(
        MessageContent::default().text("hello world"),
        &mut ("hello world".parse::<MessageContent>().unwrap())
    );
    assert_eq!(
        MessageContent::default()
            .text("hello world")
            .at(123456789)
            .at_all(),
        &mut ("hello world<@!123456789>@everyone"
            .parse::<MessageContent>()
            .unwrap())
    );
    assert_eq!(
        MessageContent::default().text(">>>>>>").at(123456789).text(" #88888 <<<<>>>>").at_all().text("<>66645"),
        &mut ("&gt;&gt;&gt;&gt;&gt;&gt;<@!123456789> #88888 &lt;&lt;&lt;&lt;&gt;&gt;&gt;&gt;@everyone&lt;&gt;66645".parse::<MessageContent>().unwrap())
    );
}
