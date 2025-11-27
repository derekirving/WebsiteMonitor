using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using System;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using Avalonia.Threading;

namespace WebsiteMonitor;

public class TrayIcon
{
    private readonly Avalonia.Controls.TrayIcon _trayIcon;
    private readonly MonitorService _monitorService;
    private Window? _mainWindow;
    private NotificationWindow? _notificationWindow;

    public TrayIcon()
    {
        _monitorService = new MonitorService();
        
        _trayIcon = new Avalonia.Controls.TrayIcon();

        var icon = new WindowIcon(
            new Bitmap(AssetLoader.Open(
                new Uri("avares://WebsiteMonitor/Assets/avalonia-logo.ico"))));
        
        _trayIcon.Icon = icon;
        _trayIcon.ToolTipText = "Website Monitor - All systems operational";
        
        var menu = new NativeMenu();
        
        var showItem = new NativeMenuItem("Show Monitor");
        showItem.Click += (s, e) => ShowMainWindow();
        menu.Add(showItem);
        
        var separator = new NativeMenuItemSeparator();
        menu.Add(separator);
        
        var exitItem = new NativeMenuItem("Exit");
        exitItem.Click += (s, e) => Exit();
        menu.Add(exitItem);
        
        _trayIcon.Menu = menu;
        _trayIcon.Clicked += (s, e) => ShowMainWindow();
        _trayIcon.IsVisible = true;
        
        _monitorService.StatusChanged += OnStatusChanged;
        _monitorService.Start();
    }

    private void OnStatusChanged(object? sender, WebsiteStatus status)
    {
        Dispatcher.UIThread.Post(() =>
        {
            if (!status.IsOnline)
            {
                _trayIcon.ToolTipText = $"Website Monitor - {status.Url} is DOWN!";
                ShowNotification($"Website Down: {status.Url}",
                    $"Status: {status.StatusCode} - {status.Message}");
            }
            else
            {
                _trayIcon.ToolTipText = "Website Monitor - All systems operational";
            }
        });
    }

    private void ShowNotification(string title, string message)
    {
        if (_notificationWindow != null && _notificationWindow.IsVisible)
        {
            _notificationWindow.Close();
        }
        
        _notificationWindow = new NotificationWindow(title, message);
        _notificationWindow.Show();
    }

    private void ShowMainWindow()
    {
        if (_mainWindow == null)
        {
            _mainWindow = new MainWindow(_monitorService);
            _mainWindow.Closed += (s, e) => _mainWindow = null;
        }
        
        _mainWindow.Show();
        _mainWindow.Activate();
    }

    private void Exit()
    {
        _monitorService.Stop();
        _trayIcon.IsVisible = false;
        
        if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime lifetime)
        {
            lifetime.Shutdown();
        }
    }
}