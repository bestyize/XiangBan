using System.Text.Json;
using Microsoft.Windows.Widgets.Providers;

namespace Xiaoban.LockScreenWidget;

internal sealed class WeatherProvider : IWidgetProvider
{
    private const string Definition = "XiaobanWeatherPrototype";
    private const string Template = """
        {
          "type": "AdaptiveCard",
          "version": "1.5",
          "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
          "body": [
            { "type": "TextBlock", "text": "小伴天气", "weight": "Bolder", "wrap": true },
            { "type": "TextBlock", "text": "18°C", "size": "Large", "weight": "Bolder", "spacing": "Small" },
            { "type": "TextBlock", "text": "${detail}", "isSubtle": true, "size": "Small", "wrap": true }
          ]
        }
        """;

    public void CreateWidget(WidgetContext context) => Publish(context);
    public void Activate(WidgetContext context) => Publish(context);
    public void OnWidgetContextChanged(WidgetContextChangedArgs args) => Publish(args.WidgetContext);
    public void DeleteWidget(string widgetId, string customState) => Program.NoteActivity();
    public void Deactivate(string widgetId) => Program.NoteActivity();
    public void OnActionInvoked(WidgetActionInvokedArgs args) => Program.NoteActivity();

    private static void Publish(WidgetContext context)
    {
        Program.NoteActivity();
        if (context.DefinitionId != Definition) return;
        // A visible timestamp distinguishes an actual host callback from a cached card.
        string detail = $"示例数据 · {DateTimeOffset.Now:HH:mm}";
        WidgetManager.GetDefault().UpdateWidget(new WidgetUpdateRequestOptions(context.Id)
        {
            Template = Template,
            Data = JsonSerializer.Serialize(new { detail }),
        });
    }
}
