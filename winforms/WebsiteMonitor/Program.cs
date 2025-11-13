namespace WebsiteMonitor;

public class MonitorApp : ApplicationContext
{
    private readonly NotifyIcon trayIcon;
    private System.Threading.Timer? pingTimer;
    private static readonly HttpClient httpClient = CreateHttpClient();
    private string targetUrl = "https://strathclyde.in/jan2026";
    private int pingIntervalSeconds = 10;
    private bool lastCheckSuccessful = true;

    public MonitorApp()
    {
        // Create tray icon
        trayIcon = new NotifyIcon()
        {
            Icon = SystemIcons.Application,
            ContextMenuStrip = CreateContextMenu(),
            Visible = true,
            Text = "Website Monitor"
        };

        trayIcon.DoubleClick += OnTrayIconDoubleClick;

        // Start monitoring
        StartMonitoring();
    }

    private static HttpClient CreateHttpClient()
    {
        var handler = new SocketsHttpHandler
        {
            // Refresh pooled connections every 5 minutes to avoid long-lived DNS issues
            PooledConnectionLifetime = TimeSpan.FromMinutes(5)
        };

        return new HttpClient(handler)
        {
            Timeout = TimeSpan.FromSeconds(10)
        };
    }

    private ContextMenuStrip CreateContextMenu()
    {
        var menu = new ContextMenuStrip();

        var urlItem = new ToolStripMenuItem($"Monitoring: {targetUrl}")
        {
            Enabled = false
        };
        menu.Items.Add(urlItem);
        
        menu.Items.Add(new ToolStripSeparator());
        
        var checkNowItem = new ToolStripMenuItem("Check Now");
        checkNowItem.Click += async (s, e) => await CheckWebsite();
        menu.Items.Add(checkNowItem);
        
        var settingsItem = new ToolStripMenuItem("Settings");
        settingsItem.Click += OnSettings;
        menu.Items.Add(settingsItem);
        
        menu.Items.Add(new ToolStripSeparator());
        
        var exitItem = new ToolStripMenuItem("Exit");
        exitItem.Click += OnExit;
        menu.Items.Add(exitItem);

        return menu;
    }

    private void StartMonitoring()
    {
        // Check immediately on start
        Task.Run(async () => await CheckWebsite());

        // Set up periodic checks
        pingTimer = new System.Threading.Timer(
            async _ => await CheckWebsite(),
            null,
            TimeSpan.FromSeconds(pingIntervalSeconds),
            TimeSpan.FromSeconds(pingIntervalSeconds)
        );
    }

    private async Task CheckWebsite()
    {
        try
        {
            var response = await httpClient.GetAsync(targetUrl);
            
            if (response.IsSuccessStatusCode)
            {
                if (!lastCheckSuccessful)
                {
                    ShowNotification("Website is back online", 
                        $"{targetUrl} is now accessible", 
                        ToolTipIcon.Info);
                }
                lastCheckSuccessful = true;
                UpdateTrayIcon(true);
            }
            else
            {
                HandleFailure($"HTTP {(int)response.StatusCode}: {response.ReasonPhrase}");
            }
        }
        catch (HttpRequestException ex)
        {
            HandleFailure($"Connection error: {ex.Message}");
        }
        catch (TaskCanceledException)
        {
            HandleFailure("Request timed out");
        }
        catch (Exception ex)
        {
            HandleFailure($"Error: {ex.Message}");
        }
    }

    private void HandleFailure(string reason)
    {
        if (lastCheckSuccessful)
        {
            ShowNotification("Website is down!", 
                $"{targetUrl}\n{reason}", 
                ToolTipIcon.Error);
        }
        lastCheckSuccessful = false;
        UpdateTrayIcon(false);
    }

    private void ShowNotification(string title, string message, ToolTipIcon icon)
    {
        trayIcon.BalloonTipTitle = title;
        trayIcon.BalloonTipText = message;
        trayIcon.BalloonTipIcon = icon;
        trayIcon.ShowBalloonTip(5000);
    }

    private void UpdateTrayIcon(bool isOnline)
    {
        // Update tooltip
        trayIcon.Text = isOnline 
            ? $"Website Monitor - Online\n{targetUrl}" 
            : $"Website Monitor - OFFLINE\n{targetUrl}";
    }

    private void OnTrayIconDoubleClick(object? sender, EventArgs? e)
    {
        ShowNotification("Website Monitor", 
            $"Monitoring: {targetUrl}\nStatus: {(lastCheckSuccessful ? "Online" : "OFFLINE")}", 
            ToolTipIcon.Info);
    }

    private void OnSettings(object? sender, EventArgs? e)
    {
        using var settingsForm = new SettingsForm(targetUrl, pingIntervalSeconds);
        if (settingsForm.ShowDialog() == DialogResult.OK)
        {
            targetUrl = settingsForm.Url;
            pingIntervalSeconds = settingsForm.IntervalSeconds;

            // Restart monitoring with new settings
            pingTimer?.Dispose();
            StartMonitoring();

            // Update context menu
            if (trayIcon.ContextMenuStrip != null)
            {
                trayIcon.ContextMenuStrip.Dispose();
                trayIcon.ContextMenuStrip = CreateContextMenu();
            }
        }
    }

    private void OnExit(object? sender, EventArgs? e)
    {
        pingTimer?.Dispose();
        httpClient?.Dispose();
        trayIcon.Visible = false;
        trayIcon.Dispose();
        Application.Exit();
    }
}

public class SettingsForm : Form
{
    private readonly TextBox urlTextBox;
    private readonly NumericUpDown intervalNumeric;
    public string Url { get; private set; } = string.Empty;
    public int IntervalSeconds { get; private set; }

    public SettingsForm(string currentUrl, int currentInterval)
    {
        Text = "Settings";
        Size = new Size(400, 180);
        StartPosition = FormStartPosition.CenterScreen;
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = false;

        var urlLabel = new Label
        {
            Text = "Website URL:",
            Location = new Point(20, 20),
            AutoSize = true
        };

        urlTextBox = new TextBox
        {
            Text = currentUrl,
            Location = new Point(20, 45),
            Width = 340
        };

        var intervalLabel = new Label
        {
            Text = "Check interval (seconds):",
            Location = new Point(20, 75),
            AutoSize = true
        };

        intervalNumeric = new NumericUpDown
        {
            Value = currentInterval,
            Minimum = 10,
            Maximum = 3600,
            Location = new Point(20, 100),
            Width = 100
        };

        var okButton = new Button
        {
            Text = "OK",
            DialogResult = DialogResult.OK,
            Location = new Point(200, 110),
            Width = 75
        };

        var cancelButton = new Button
        {
            Text = "Cancel",
            DialogResult = DialogResult.Cancel,
            Location = new Point(285, 110),
            Width = 75
        };

        okButton.Click += (s, e) =>
        {
            Url = urlTextBox.Text;
            IntervalSeconds = (int)intervalNumeric.Value;
        };

        Controls.AddRange([ 
            urlLabel, urlTextBox, 
            intervalLabel, intervalNumeric, 
            okButton, cancelButton 
        ]);

        AcceptButton = okButton;
        CancelButton = cancelButton;
    }
}

static class Program
{
    [STAThread]
    static void Main()
    {
        Application.EnableVisualStyles();
        Application.SetCompatibleTextRenderingDefault(false);
        Application.Run(new MonitorApp());
    }
}