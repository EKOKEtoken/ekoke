import * as React from 'react';
import { useConnectedMetaMask } from 'metamask-react';

import Web3Client from '../../../web3/Web3Client';
import { ChainId } from '../../MetamaskConnect';
import { e8sToEkoke } from '../../../utils';
import Container from '../../reusable/Container';
import Heading from '../../reusable/Heading';
import Hr from '../../reusable/Hr';
import Link from '../../reusable/Link';
import { CONTRACT_ADDRESS } from '../../../web3/contracts/Ekoke';
import Button from '../../reusable/Button';
import { Page, PageProps } from '../ConnectedPage';
import EkokeLogo from '../../../../assets/images/ekoke-logo.webp';
import Paragraph from '../../reusable/Paragraph';

const Summary = ({ onSwitchPage }: PageProps) => {
  const { account, ethereum, chainId } = useConnectedMetaMask();
  const [swappedSupply, setSwappedSupply] = React.useState<string>('');

  React.useEffect(() => {
    if (account && chainId) {
      const client = new Web3Client(account, ethereum, chainId as ChainId);
      client.swappedSupply().then((supply) => {
        setSwappedSupply(e8sToEkoke(supply));
      });
    }
  }, [account, chainId]);

  return (
    <Container.FlexCols className="items-center justify-center">
      <Container.Card className="px-12 sm:px-1">
        <Container.FlexCols className="items-center justify-center">
          <Heading.H1>EKOKE Token</Heading.H1>
          <img
            src={EkokeLogo}
            alt="Ekoke Logo"
            className="h-[128px] sm:h-[64px] mr-4"
          />
        </Container.FlexCols>
        <Hr />
        <Container.FlexCols className="gap-4">
          <Container.Container>
            <Paragraph.Default className="sm:!w-min text-lg sm:text-xs !text-left">
              ERC20 Token Address:{' '}
              <Link.Default
                className="sm:text-xs"
                href={`https://etherscan.io/address/${
                  CONTRACT_ADDRESS[chainId as ChainId]
                }`}
              >
                {CONTRACT_ADDRESS[chainId as ChainId]}
              </Link.Default>
            </Paragraph.Default>
          </Container.Container>
          <Container.Container>
            <Paragraph.Default className="sm:!w-min text-lg sm:text-xs !text-left">
              ERC20 Swapped Supply: {swappedSupply}
            </Paragraph.Default>
          </Container.Container>
          <Container.FlexResponsiveRow className="items-center justify-center gap-8 sm:gap-2">
            <Button.Cta onClick={() => onSwitchPage(Page.IcrcToErc20)}>
              <span>Swap ICRC into ERC20</span>
            </Button.Cta>
            <Button.Cta onClick={() => onSwitchPage(Page.Erc20ToIcrc)}>
              <span>Swap ERC20 into ICRC</span>
            </Button.Cta>
          </Container.FlexResponsiveRow>
        </Container.FlexCols>
      </Container.Card>
    </Container.FlexCols>
  );
};

export default Summary;
