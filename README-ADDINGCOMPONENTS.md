# Adding Components

Adding a new component is relatively trivial and involves a few simple steps; though requires some upfront plumbing. This document should hopefully help make this process easier.

## Adding a Config

First add a config to the `GlazeWM.Domain.UserConfigs` namespace; here is an example: 

```csharp
public class CpuPercentComponentConfig : BarComponentConfig
{
    /// <summary>
    /// Label/icon assigned to the CPU component.
    /// {0} is substituted by CPU percentage formatted using <see cref="NumberFormat"/>.
    /// </summary>
    public string StringFormat { get; set; } = "CPU {0}%";

    /// <summary>
    /// Numerical Format to use for the Percentage.
    /// </summary>
    public string NumberFormat { get; set; } = "00";
}
```

Once this is done, you should register this config in `BarComponentConfigConverter.Read`:  

```csharp
"cpu percent" =>
  JsonSerializer.Deserialize<CpuPercentComponentConfig>(
    jsonObject.RootElement.ToString(),
    options
  )
```

This will map directly to the config using the YAML naming rules, i.e.

```yaml
components_right:
  - type: "cpu percent"
    string_format: "CPU {0}%" # {0} is substituted by number format
    number_format: "00" # See https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-numeric-format-strings#standard-format-specifiers.
```

Each capital letter is prefixed by a space delimited by `_`.  

## Adding a Service [Optional]

In some cases to add a component, you might want to register a `'Service'`; this will allow your logic to be reused across multiple locations in the program.

To do so, first create the raw service in `GlazeWM.Infrastructure`.  

```csharp
/// <summary>
/// Provides access to current CPU statistics.
/// </summary>
public class CpuStatsService : System.IDisposable
{
  private readonly PerformanceCounter _cpuCounter = new("Processor Information", "% Processor Utility", "_Total");

  /// <inheritdoc />
  ~CpuStatsService() => Dispose();

  /// <inheritdoc />
  public void Dispose()
  {
    _cpuCounter?.Dispose();
    GC.SuppressFinalize(this);
  }

  /// <summary>
  /// Returns the current CPU utilization as a percentage.
  /// </summary>
  public float GetCurrentLoadPercent() => _cpuCounter.NextValue();
}
```

And register it in the DI container in `GlazeWM.Infrastructure.DependencyInjection`:  

```csharp
services.AddSingleton<CpuStatsService>();
```

Note: I added cleanup/dispose code here because it is good practice; it is not technically necessary if the service is a singleton.

## Adding a ViewModel

This contains the main logic used to fetch the data to be displayed in the UI.  
You add this file to `GlazeWM.Bar.Components`.  

Example:

```csharp
public class CpuPercentComponentViewModel : ComponentViewModel
{
  private CpuPercentComponentConfig Config => _componentConfig as CpuPercentComponentConfig;
  private readonly CpuStatsService _cpuStatsService;

  public string FormattedText => GetFormattedText();

  public CpuPercentComponentViewModel(BarViewModel parentViewModel, CpuPercentComponentConfig config) : base(parentViewModel, config)
  {
    // Get the service from DI
    _cpuStatsService = ServiceLocator.GetRequiredService<CpuStatsService>();

    var updateInterval = TimeSpan.FromSeconds(1);
    Observable
      .Interval(updateInterval)
      .TakeUntil(_parentViewModel.WindowClosing)
      .Subscribe(_ => OnPropertyChanged(nameof(FormattedText)));
  }

  private string GetFormattedText()
  {
    var percent = _cpuStatsService.GetCurrentLoadPercent().ToString(Config.NumberFormat, CultureInfo.InvariantCulture);
    return string.Format(CultureInfo.InvariantCulture, Config.StringFormat, percent);
  }
}
```

Note the line `.TakeUntil(_parentViewModel.WindowClosing)`. This is necessary to correctly clean up after config reloads; otherwise your component will remain alive; wasting CPU and RAM.

Then register your ViewModel to `BarViewModel.CreateComponentViewModels`

```csharp
CpuPercentComponentConfig cpupc => new CpuPercentComponentViewModel(this, cpupc),
```

## Adding a Component XAML

Each individual component has its own custom XAML WPF UserControl.  
Most of these don't require much customization; so it's usually easier to just copy an existing component; for example.  

### Copy Existing Component

CTRL+C & CTRL+V existing Xaml file

![Duplicate XAML](./docs/images/duplicate_xaml.png)

### Fix Class Name

Fix the class name to correspond to your new class in the XAML, i.e.

```xaml
<UserControl
  x:Class="GlazeWM.Bar.Components.CpuPercentComponent"
```

(first line)

And corresponding `xaml.cs` file.

```csharp
public partial class CpuPercentComponent : UserControl
{
  public CpuPercentComponent() => InitializeComponent();
}
```

### Fix Item Name

Change following line in the XAML

```xaml
<!-- Or whichever property is here -->
Text="{Binding TilingDirectionString}" /> 
```

To match your property from the `ViewModel`, i.e. `FormattedText`, so the XAML reads.

```xaml
Text="{Binding FormattedText}" />
```

## Register Component XAML

Add your component to `ComponentPortal.xaml` beside all the other components.

```xaml
<DataTemplate DataType="{x:Type components:CpuPercentComponentViewModel}">
  <components:CpuPercentComponent Padding="{Binding Padding}" Background="{Binding Background}" />
</DataTemplate>
```

After doing this, add the component to your config, see if it works.

![Working CPU Indicator](./docs/images/working_cpu_indicator.png)