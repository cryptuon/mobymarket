import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import type { WalletInfo, WalletProvider } from '@/types'

export const useWalletStore = defineStore('wallet', () => {
  // State
  const isConnecting = ref(false)
  const isConnected = ref(false)
  const address = ref<string>('')
  const chainId = ref<number>(1)
  const balance = ref<string>('0')
  const provider = ref<WalletProvider | null>(null)
  const walletType = ref<string>('')

  // Getters
  const walletInfo = computed<WalletInfo>(() => ({
    address: address.value,
    chainId: chainId.value,
    isConnected: isConnected.value,
    balance: balance.value,
    walletType: walletType.value,
  }))

  const formattedAddress = computed(() => {
    if (!address.value) return ''
    return `${address.value.slice(0, 6)}...${address.value.slice(-4)}`
  })

  const networkName = computed(() => {
    switch (chainId.value) {
      case 1: return 'Ethereum Mainnet'
      case 5: return 'Goerli Testnet'
      case 137: return 'Polygon'
      case 80001: return 'Polygon Mumbai'
      case 42161: return 'Arbitrum One'
      case 421613: return 'Arbitrum Goerli'
      case 10: return 'Optimism'
      case 420: return 'Optimism Goerli'
      case 8453: return 'Base'
      case 84531: return 'Base Goerli'
      default: return 'Unknown Network'
    }
  })

  const isMainnet = computed(() => {
    return [1, 137, 42161, 10, 8453].includes(chainId.value)
  })

  // Actions
  async function connect(walletProvider?: string): Promise<void> {
    if (isConnecting.value) return

    isConnecting.value = true

    try {
      // Check if ethereum object exists
      if (typeof window === 'undefined' || !window.ethereum) {
        throw new Error('No wallet provider found. Please install MetaMask or another Web3 wallet.')
      }

      // Request account access
      const accounts = await window.ethereum.request({
        method: 'eth_requestAccounts'
      })

      if (accounts.length === 0) {
        throw new Error('No accounts found. Please unlock your wallet.')
      }

      // Get chain ID
      const networkId = await window.ethereum.request({
        method: 'eth_chainId'
      })

      // Set wallet state
      address.value = accounts[0]
      chainId.value = parseInt(networkId, 16)
      isConnected.value = true
      walletType.value = walletProvider || detectWalletType()

      // Get initial balance
      await updateBalance()

      // Set up event listeners
      setupEventListeners()

    } catch (error) {
      console.error('Failed to connect wallet:', error)
      throw error
    } finally {
      isConnecting.value = false
    }
  }

  async function disconnect(): Promise<void> {
    // Clear state
    address.value = ''
    chainId.value = 1
    balance.value = '0'
    isConnected.value = false
    walletType.value = ''
    provider.value = null

    // Remove event listeners
    removeEventListeners()
  }

  async function switchChain(targetChainId: number): Promise<void> {
    if (!window.ethereum) {
      throw new Error('No wallet provider found')
    }

    try {
      await window.ethereum.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: `0x${targetChainId.toString(16)}` }],
      })
    } catch (error: any) {
      // If the chain hasn't been added to the wallet
      if (error.code === 4902) {
        await addChain(targetChainId)
      } else {
        throw error
      }
    }
  }

  async function addChain(chainId: number): Promise<void> {
    if (!window.ethereum) {
      throw new Error('No wallet provider found')
    }

    const chainConfig = getChainConfig(chainId)
    if (!chainConfig) {
      throw new Error(`Unsupported chain ID: ${chainId}`)
    }

    await window.ethereum.request({
      method: 'wallet_addEthereumChain',
      params: [chainConfig],
    })
  }

  async function updateBalance(): Promise<void> {
    if (!address.value || !window.ethereum) return

    try {
      const balanceHex = await window.ethereum.request({
        method: 'eth_getBalance',
        params: [address.value, 'latest']
      })

      // Convert from wei to ether
      const balanceWei = parseInt(balanceHex, 16)
      const balanceEther = balanceWei / Math.pow(10, 18)
      balance.value = balanceEther.toString()
    } catch (error) {
      console.error('Failed to update balance:', error)
    }
  }

  async function signMessage(message: string): Promise<string> {
    if (!address.value || !window.ethereum) {
      throw new Error('Wallet not connected')
    }

    return await window.ethereum.request({
      method: 'personal_sign',
      params: [message, address.value]
    })
  }

  // Helper functions
  function detectWalletType(): string {
    if (window.ethereum?.isMetaMask) return 'MetaMask'
    if (window.ethereum?.isCoinbaseWallet) return 'Coinbase Wallet'
    if (window.ethereum?.isRabby) return 'Rabby'
    if (window.ethereum?.isBraveWallet) return 'Brave Wallet'
    return 'Unknown'
  }

  function setupEventListeners(): void {
    if (!window.ethereum) return

    window.ethereum.on('accountsChanged', handleAccountsChanged)
    window.ethereum.on('chainChanged', handleChainChanged)
    window.ethereum.on('disconnect', handleDisconnect)
  }

  function removeEventListeners(): void {
    if (!window.ethereum) return

    window.ethereum.removeListener('accountsChanged', handleAccountsChanged)
    window.ethereum.removeListener('chainChanged', handleChainChanged)
    window.ethereum.removeListener('disconnect', handleDisconnect)
  }

  function handleAccountsChanged(accounts: string[]): void {
    if (accounts.length === 0) {
      disconnect()
    } else if (accounts[0] !== address.value) {
      address.value = accounts[0]
      updateBalance()
    }
  }

  function handleChainChanged(newChainId: string): void {
    chainId.value = parseInt(newChainId, 16)
    updateBalance()
  }

  function handleDisconnect(): void {
    disconnect()
  }

  function getChainConfig(chainId: number) {
    const configs: Record<number, any> = {
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
          name: 'ETH',
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
          name: 'ETH',
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
          name: 'ETH',
          symbol: 'ETH',
          decimals: 18
        },
        rpcUrls: ['https://mainnet.base.org/'],
        blockExplorerUrls: ['https://basescan.org/']
      }
    }

    return configs[chainId]
  }

  // Auto-connect on page load if previously connected
  async function autoConnect(): Promise<void> {
    if (typeof window === 'undefined' || !window.ethereum) return

    try {
      const accounts = await window.ethereum.request({
        method: 'eth_accounts'
      })

      if (accounts.length > 0) {
        await connect()
      }
    } catch (error) {
      console.error('Auto-connect failed:', error)
    }
  }

  return {
    // State
    isConnecting,
    isConnected,
    address,
    chainId,
    balance,
    provider,
    walletType,

    // Getters
    walletInfo,
    formattedAddress,
    networkName,
    isMainnet,

    // Actions
    connect,
    disconnect,
    switchChain,
    addChain,
    updateBalance,
    signMessage,
    autoConnect,
  }
})

// Global type extension for window.ethereum
declare global {
  interface Window {
    ethereum?: {
      isMetaMask?: boolean
      isCoinbaseWallet?: boolean
      isRabby?: boolean
      isBraveWallet?: boolean
      request: (args: { method: string; params?: any[] }) => Promise<any>
      on: (event: string, handler: (...args: any[]) => void) => void
      removeListener: (event: string, handler: (...args: any[]) => void) => void
    }
  }
}