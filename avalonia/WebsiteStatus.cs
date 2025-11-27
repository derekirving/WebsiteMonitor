using System;
using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace WebsiteMonitor;

public class WebsiteStatus : INotifyPropertyChanged
{
    private string? _url;
    private string? _statusText;
    private int _statusCode;
    private string? _message;
    private string? _lastCheckedText;
    private bool _isOnline = true;
    private DateTime? _lastChecked;

    public string? Url { get => _url; set => Set(ref _url, value); }
    public string? StatusText { get => _statusText; set => Set(ref _statusText, value); }
    public int StatusCode { get => _statusCode; set => Set(ref _statusCode, value); }
    
    public bool IsOnline { get => _isOnline; set => Set(ref _isOnline, value); }
    public DateTime? LastChecked { get => _lastChecked; set => Set(ref _lastChecked, value); }
    public string? Message { get => _message; set => Set(ref _message, value); }
    public string? LastCheckedText { get => _lastCheckedText; set => Set(ref _lastCheckedText, value); }
    
    

    public event PropertyChangedEventHandler? PropertyChanged;
    protected bool Set<T>(ref T field, T value, [CallerMemberName] string? propName = null)
    {
        if (Equals(field, value)) return false;
        field = value;
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propName));
        return true;
    }
}

// public class WebsiteStatus : ViewModelBase
// {
//     private string _url = string.Empty;
//     private bool _isOnline = true;
//     private int _statusCode;
//     private string _message = "Not checked yet";
//     private DateTime? _lastChecked;
//
//     public string Url
//     {
//         get => _url;
//         set
//         {
//             if (_url != value)
//             {
//                 _url = value;
//                 OnPropertyChanged();
//             }
//         }
//     }
//
//     public bool IsOnline
//     {
//         get => _isOnline;
//         set
//         {
//             if (_isOnline != value)
//             {
//                 _isOnline = value;
//                 OnPropertyChanged();
//                 OnPropertyChanged(nameof(StatusText));
//             }
//         }
//     }
//
//     public int StatusCode
//     {
//         get => _statusCode;
//         set
//         {
//             if (_statusCode != value)
//             {
//                 _statusCode = value;
//                 OnPropertyChanged();
//             }
//         }
//     }
//
//     public string Message
//     {
//         get => _message;
//         set
//         {
//             if (_message != value)
//             {
//                 _message = value;
//                 OnPropertyChanged();
//             }
//         }
//     }
//
//     public DateTime? LastChecked
//     {
//         get => _lastChecked;
//         set
//         {
//             if (_lastChecked != value)
//             {
//                 _lastChecked = value;
//                 OnPropertyChanged();
//                 OnPropertyChanged(nameof(LastCheckedText));
//             }
//         }
//     }
//
//     public string StatusText
//     {
//         get => IsOnline ? "✓ Online" : "✗ Offline";
//         init => throw new NotImplementedException();
//     }
//
//     public string LastCheckedText => LastChecked?.ToString("HH:mm:ss") ?? "Never";
// }