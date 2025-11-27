using Avalonia.Controls;
using Avalonia.Interactivity;
using System;
using System.Collections.ObjectModel;

namespace WebsiteMonitor;

public partial class MainWindow : Window
{
    private MonitorService? _monitorService;
    

    public MainWindow() : this(null!)
    {
        InitializeComponent();
    }

    public MainWindow(MonitorService monitorService)
    {
        InitializeComponent();
        _monitorService = monitorService;
        
        if (_monitorService != null)
        {
            var dataGrid = this.FindControl<DataGrid>("WebsiteDataGrid");
            if (dataGrid != null)
            {
                dataGrid.ItemsSource = _monitorService.Websites;
            }
        }
    }

    private void AddButton_Click(object? sender, RoutedEventArgs e)
    {
        if (_monitorService == null) return;
        
        
        var urlTextBox = this.FindControl<TextBox>("UrlTextBox");
        if (urlTextBox?.Text != null && !string.IsNullOrWhiteSpace(urlTextBox.Text))
        {
            var url = urlTextBox.Text.Trim();
            
            if (!url.StartsWith("http://") && !url.StartsWith("https://"))
            {
                url = "https://" + url;
            }
        
            if (Uri.TryCreate(url, UriKind.Absolute, out _))
            {
                _monitorService.AddWebsite(url);
                urlTextBox.Text = string.Empty;
            }
        }
    }

    private void RemoveButton_Click(object? sender, RoutedEventArgs e)
    {
        if (_monitorService == null) return;
        
        if (sender is Button button && button.Tag is WebsiteStatus website)
        {
            _monitorService.RemoveWebsite(website);
        }
    }
}