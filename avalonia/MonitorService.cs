using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;

namespace WebsiteMonitor;

public class MonitorService
{
    private readonly HttpClient _httpClient;
    private readonly ObservableCollection<WebsiteStatus> _websites;
    private Timer? _timer;
    private readonly int _checkIntervalSeconds = 60;
    
    public event EventHandler<WebsiteStatus>? StatusChanged;
    public ObservableCollection<WebsiteStatus> Websites => _websites;

    public MonitorService()
    {
        _httpClient = new HttpClient
        {
            Timeout = TimeSpan.FromSeconds(10)
        };
        
        _websites = new ObservableCollection<WebsiteStatus>
        {
            new WebsiteStatus { Url = "https://www.fgoogle.com" },
            new WebsiteStatus { Url = "https://www.github.com" },
            new WebsiteStatus { Url = "https://www.microsoft.com" }
        };
    }

    public void Start()
    {
        _timer = new Timer(CheckWebsites, null, TimeSpan.Zero, TimeSpan.FromSeconds(_checkIntervalSeconds));
    }

    public void Stop()
    {
        _timer?.Dispose();
        _timer = null;
    }

    public void AddWebsite(string url)
    {
        if (!_websites.Any(w => w.Url.Equals(url, StringComparison.OrdinalIgnoreCase)))
        {
            _websites.Add(new WebsiteStatus { Url = url });
        }
    }

    public void RemoveWebsite(WebsiteStatus website)
    {
        _websites.Remove(website);
    }

    private async void CheckWebsites(object? state)
    {
        try
        {
            var tasks = _websites.Select(CheckWebsite).ToList();
            await Task.WhenAll(tasks);
        }
        catch (Exception e)
        {
            throw; // TODO handle exception
        }
    }

    private async Task CheckWebsite(WebsiteStatus website)
    {
        var previousStatus = website.IsOnline;
        
        try
        {
            var response = await _httpClient.GetAsync(website.Url);
            
            website.StatusCode = (int)response.StatusCode;
            website.IsOnline = response.IsSuccessStatusCode;
            website.Message = response.IsSuccessStatusCode 
                ? "OK" 
                : $"Status: {response.StatusCode}";
            website.LastChecked = DateTime.Now;
            
            if (previousStatus && !website.IsOnline)
            {
                StatusChanged?.Invoke(this, website);
            }
        }
        catch (TaskCanceledException)
        {
            website.StatusCode = 0;
            website.IsOnline = false;
            website.Message = "Timeout";
            website.LastChecked = DateTime.Now;
            
            if (previousStatus)
            {
                StatusChanged?.Invoke(this, website);
            }
        }
        catch (HttpRequestException ex)
        {
            website.StatusCode = 0;
            website.IsOnline = false;
            website.Message = ex.Message;
            website.LastChecked = DateTime.Now;
            
            if (previousStatus)
            {
                StatusChanged?.Invoke(this, website);
            }
        }
        catch (Exception ex)
        {
            website.StatusCode = 0;
            website.IsOnline = false;
            website.Message = ex.Message;
            website.LastChecked = DateTime.Now;
            
            if (previousStatus)
            {
                StatusChanged?.Invoke(this, website);
            }
        }
    }
}