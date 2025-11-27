using Avalonia;
using Avalonia.Controls;
using Avalonia.Threading;
using System;

namespace WebsiteMonitor;

public partial class NotificationWindow : Window
{
    private DispatcherTimer? _timer;

    public NotificationWindow() : this(string.Empty, string.Empty)
    {
        InitializeComponent();
    }

    public NotificationWindow(string title, string message)
    {
        InitializeComponent();
        
        var titleText = this.FindControl<TextBlock>("TitleText");
        var messageText = this.FindControl<TextBlock>("MessageText");
        
        if (titleText != null) titleText.Text = title;
        if (messageText != null) messageText.Text = message;
        
        // Position in bottom right corner
        PositionWindow();
        
        // Auto-close after 5 seconds
        _timer = new DispatcherTimer
        {
            Interval = TimeSpan.FromSeconds(5)
        };
        _timer.Tick += (s, e) =>
        {
            _timer?.Stop();
            Close();
        };
        _timer.Start();
        
        // Close on click
        PointerPressed += (s, e) => Close();
    }

    private void PositionWindow()
    {
        if (Screens.Primary != null)
        {
            var screen = Screens.Primary.WorkingArea;
            Position = new PixelPoint(
                screen.Right - (int)Width - 20,
                screen.Bottom - (int)Height - 20
            );
        }
    }

    protected override void OnClosed(EventArgs e)
    {
        _timer?.Stop();
        _timer = null;
        base.OnClosed(e);
    }
}