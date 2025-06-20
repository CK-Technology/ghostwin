# Get the currently active network connection
$currentNetwork = Get-NetConnectionProfile | Where-Object { $_.IPv4Connectivity -eq "Internet" }

# Change the network category to Private if it is set to Public
if ($currentNetwork.NetworkCategory -eq "Public") {
    Set-NetConnectionProfile -InterfaceAlias $currentNetwork.InterfaceAlias -NetworkCategory Private
}
