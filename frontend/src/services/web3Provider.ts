import type { WalletProvider } from '@/types'

export class Web3ProviderService {
  private static instance: Web3ProviderService
  private providers: Map<string, WalletProvider> = new Map()

  private constructor() {
    this.detectProviders()
  }

  static getInstance(): Web3ProviderService {
    if (!Web3ProviderService.instance) {
      Web3ProviderService.instance = new Web3ProviderService()
    }
    return Web3ProviderService.instance
  }

  private detectProviders() {
    // MetaMask
    if (window.ethereum?.isMetaMask) {
      this.providers.set('metamask', {
        name: 'MetaMask',
        isInstalled: true,
        icon: '/icons/metamask.svg',
        connector: window.ethereum
      })
    }

    // Coinbase Wallet
    if (window.ethereum?.isCoinbaseWallet) {
      this.providers.set('coinbase', {
        name: 'Coinbase Wallet',
        isInstalled: true,
        icon: '/icons/coinbase.svg',
        connector: window.ethereum
      })
    }

    // WalletConnect (would need additional setup)
    this.providers.set('walletconnect', {
      name: 'WalletConnect',
      isInstalled: true, // Always available
      icon: '/icons/walletconnect.svg',
      connector: null // Would be initialized with WalletConnect
    })

    // Rabby
    if (window.ethereum?.isRabby) {
      this.providers.set('rabby', {
        name: 'Rabby',
        isInstalled: true,
        icon: '/icons/rabby.svg',
        connector: window.ethereum
      })
    }

    // Brave Wallet
    if (window.ethereum?.isBraveWallet) {
      this.providers.set('brave', {
        name: 'Brave Wallet',
        isInstalled: true,
        icon: '/icons/brave.svg',
        connector: window.ethereum
      })
    }

    // Add fallback providers that aren't installed
    this.addFallbackProviders()
  }

  private addFallbackProviders() {
    const fallbackProviders = [
      {
        id: 'metamask',
        name: 'MetaMask',
        icon: '/icons/metamask.svg',
        downloadUrl: 'https://metamask.io/download/'
      },
      {
        id: 'coinbase',
        name: 'Coinbase Wallet',
        icon: '/icons/coinbase.svg',
        downloadUrl: 'https://wallet.coinbase.com/'
      },
      {
        id: 'rainbow',
        name: 'Rainbow',
        icon: '/icons/rainbow.svg',
        downloadUrl: 'https://rainbow.me/'
      },
      {
        id: 'trust',
        name: 'Trust Wallet',
        icon: '/icons/trust.svg',
        downloadUrl: 'https://trustwallet.com/'
      }
    ]

    for (const provider of fallbackProviders) {
      if (!this.providers.has(provider.id)) {
        this.providers.set(provider.id, {
          name: provider.name,
          isInstalled: false,
          icon: provider.icon,
          connector: null
        })
      }
    }
  }

  getAvailableProviders(): WalletProvider[] {
    return Array.from(this.providers.values())
  }

  getInstalledProviders(): WalletProvider[] {
    return Array.from(this.providers.values()).filter(p => p.isInstalled)
  }

  getProvider(name: string): WalletProvider | undefined {
    return this.providers.get(name.toLowerCase())
  }

  async connectProvider(name: string): Promise<any> {
    const provider = this.getProvider(name)

    if (!provider) {
      throw new Error(`Provider ${name} not found`)
    }

    if (!provider.isInstalled) {
      throw new Error(`${provider.name} is not installed`)
    }

    if (!provider.connector) {
      throw new Error(`${provider.name} connector not available`)
    }

    // Request account access
    const accounts = await provider.connector.request({
      method: 'eth_requestAccounts'
    })

    if (!accounts || accounts.length === 0) {
      throw new Error('No accounts found')
    }

    return {
      connector: provider.connector,
      accounts,
      providerName: provider.name
    }
  }

  async switchNetwork(chainId: number, provider?: any): Promise<void> {
    const connector = provider || window.ethereum

    if (!connector) {
      throw new Error('No wallet provider available')
    }

    const hexChainId = `0x${chainId.toString(16)}`

    try {
      await connector.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: hexChainId }]
      })
    } catch (error: any) {
      // If chain hasn't been added yet
      if (error.code === 4902) {
        await this.addNetwork(chainId, connector)
      } else {
        throw error
      }
    }
  }

  async addNetwork(chainId: number, provider?: any): Promise<void> {
    const connector = provider || window.ethereum

    if (!connector) {
      throw new Error('No wallet provider available')
    }

    const networkConfig = this.getNetworkConfig(chainId)
    if (!networkConfig) {
      throw new Error(`Network configuration for chain ID ${chainId} not found`)
    }

    await connector.request({
      method: 'wallet_addEthereumChain',
      params: [networkConfig]
    })
  }

  private getNetworkConfig(chainId: number) {
    const networks: Record<number, any> = {
      1: {
        chainId: '0x1',
        chainName: 'Ethereum Mainnet',
        nativeCurrency: {
          name: 'Ether',
          symbol: 'ETH',
          decimals: 18
        },
        rpcUrls: ['https://mainnet.infura.io/v3/'],
        blockExplorerUrls: ['https://etherscan.io/']
      },
      137: {
        chainId: '0x89',
        chainName: 'Polygon Mainnet',
        nativeCurrency: {
          name: 'MATIC',
          symbol: 'MATIC',
          decimals: 18
        },
        rpcUrls: ['https://polygon-rpc.com/'],
        blockExplorerUrls: ['https://polygonscan.com/']
      },
      42161: {
        chainId: '0xa4b1',
        chainName: 'Arbitrum One',
        nativeCurrency: {
          name: 'Ether',
          symbol: 'ETH',
          decimals: 18
        },
        rpcUrls: ['https://arb1.arbitrum.io/rpc'],
        blockExplorerUrls: ['https://arbiscan.io/']
      },
      10: {
        chainId: '0xa',
        chainName: 'Optimism',
        nativeCurrency: {
          name: 'Ether',
          symbol: 'ETH',
          decimals: 18
        },
        rpcUrls: ['https://mainnet.optimism.io/'],
        blockExplorerUrls: ['https://optimistic.etherscan.io/']
      },
      8453: {
        chainId: '0x2105',
        chainName: 'Base',
        nativeCurrency: {
          name: 'Ether',
          symbol: 'ETH',
          decimals: 18
        },
        rpcUrls: ['https://mainnet.base.org/'],
        blockExplorerUrls: ['https://basescan.org/']
      }
    }

    return networks[chainId]
  }

  async getBalance(address: string, provider?: any): Promise<string> {
    const connector = provider || window.ethereum

    if (!connector) {
      throw new Error('No wallet provider available')
    }

    const balance = await connector.request({
      method: 'eth_getBalance',
      params: [address, 'latest']
    })

    // Convert from wei to ether
    const balanceInWei = parseInt(balance, 16)
    const balanceInEther = balanceInWei / Math.pow(10, 18)

    return balanceInEther.toString()
  }

  async signMessage(message: string, address: string, provider?: any): Promise<string> {
    const connector = provider || window.ethereum

    if (!connector) {
      throw new Error('No wallet provider available')
    }

    return await connector.request({
      method: 'personal_sign',
      params: [message, address]
    })
  }

  async getChainId(provider?: any): Promise<number> {
    const connector = provider || window.ethereum

    if (!connector) {
      throw new Error('No wallet provider available')
    }

    const chainId = await connector.request({
      method: 'eth_chainId'
    })

    return parseInt(chainId, 16)
  }

  async getAccounts(provider?: any): Promise<string[]> {
    const connector = provider || window.ethereum

    if (!connector) {
      throw new Error('No wallet provider available')
    }

    return await connector.request({
      method: 'eth_accounts'
    })
  }

  // Event listener helpers
  onAccountsChanged(callback: (accounts: string[]) => void, provider?: any) {
    const connector = provider || window.ethereum
    if (connector) {
      connector.on('accountsChanged', callback)
    }
  }

  onChainChanged(callback: (chainId: string) => void, provider?: any) {
    const connector = provider || window.ethereum
    if (connector) {
      connector.on('chainChanged', callback)
    }
  }

  onDisconnect(callback: (error: any) => void, provider?: any) {
    const connector = provider || window.ethereum
    if (connector) {
      connector.on('disconnect', callback)
    }
  }

  removeAllListeners(provider?: any) {
    const connector = provider || window.ethereum
    if (connector && typeof connector.removeAllListeners === 'function') {
      connector.removeAllListeners()
    }
  }
}

// Export singleton instance
export const web3Provider = Web3ProviderService.getInstance()