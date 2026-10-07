int main(int argc,char **argv){
 if(argc<4)return 2;
 int result=unicode_original_main(3,argv);if(result)return result;
 if(!strcmp(argv[3],"parse")){memset(unicode_db,0,sizeof(*unicode_db)*(CHARCODE_MAX+1));char filename[1024];snprintf(filename,sizeof(filename),"%s/UnicodeData.txt",argv[1]);parse_unicode_data_dormant(filename);}
 else if(!strcmp(argv[3],"conv"))build_conv_table_dormant(unicode_db,-1);
 else if(!strcmp(argv[3],"props")){compute_internal_props_dormant();uint32_t h=0;for(int i=0;i<=CHARCODE_MAX;i++)h=h*31+get_prop(i,PROP_Cased1);printf("%08x\n",h);}
 else if(!strcmp(argv[3],"mark")){REStringList sl={0};uint32_t buf[]={0x1f469,0x200d,0x1f4bb};re_string_list_init(&sl);re_string_add(&sl,3,buf);int result=mark_zwj_string_dormant(&sl,buf,3,EMOJI_MOD_NONE,NULL,-1,1);printf("%d\n",result);re_string_list_free(&sl);}
 else if(!strcmp(argv[3],"zwj")){char filename[1024];re_string_list_init(&rgi_emoji_zwj_sequence);snprintf(filename,sizeof(filename),"%s/emoji-zwj-sequences.txt",argv[1]);parse_sequence_prop_list(filename);FILE *f=fopen("/dev/null","wb");build_rgi_emoji_zwj_sequence_dormant(f,&rgi_emoji_zwj_sequence);fclose(f);}
 else if(!strcmp(argv[3],"compose")){FILE *f=fopen("/dev/null","wb");build_decompose_table_with_dormant_composition(f);fclose(f);}
 else if(!strcmp(argv[3],"large"))dump_large_char();else return 2;
 return 0;
}
